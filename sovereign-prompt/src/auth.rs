//! Shared HTTP trust boundary. Stdio does not use HTTP credentials.
use anyhow::{bail, Context, Result};
use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

pub const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub enum HttpService {
    Mcp,
    Dashboard,
    Proxy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    pub user_id: String,
}

#[derive(Clone)]
struct Credential {
    principal: Principal,
    tag: Vec<u8>,
}

#[derive(Clone)]
pub struct HttpSecurity {
    service: HttpService,
    credentials: Vec<Credential>,
    allowed_hosts: Vec<String>,
    allowed_origins: Vec<String>,
}

fn key_mac(key: &str) -> Hmac<Sha256> {
    // A fixed context key is sufficient here: only constant-time equality of
    // stored token digests is needed, not a password derivation function.
    let mut mac = Hmac::<Sha256>::new_from_slice(b"SovereignPrompt HTTP token comparison v1")
        .expect("HMAC accepts any key size");
    mac.update(key.as_bytes());
    mac
}

impl HttpSecurity {
    pub fn from_env(bind: SocketAddr, service: HttpService) -> Result<Self> {
        let mut keys = match std::env::var("SOVEREIGN_API_KEYS") {
            Ok(value) => serde_json::from_str::<BTreeMap<String, String>>(&value)
                .context("SOVEREIGN_API_KEYS must map user IDs to API keys")?,
            Err(std::env::VarError::NotPresent) => BTreeMap::new(),
            Err(error) => return Err(error.into()),
        };
        match std::env::var("SOVEREIGN_API_KEY") {
            Ok(key) => {
                let user = match std::env::var("SOVEREIGN_USER_ID") {
                    Ok(user) => user,
                    Err(std::env::VarError::NotPresent) => "anonymous".into(),
                    Err(error) => return Err(error.into()),
                };
                if keys.insert(user, key).is_some() {
                    bail!("SOVEREIGN_API_KEY duplicates a user in SOVEREIGN_API_KEYS");
                }
            }
            Err(std::env::VarError::NotPresent) => {}
            Err(error) => return Err(error.into()),
        }
        let hosts = read_list("SOVEREIGN_ALLOWED_HOSTS")?;
        let origins = read_list("SOVEREIGN_ALLOWED_ORIGINS")?;
        Self::new(bind, service, keys, hosts, origins)
    }

    /// Explicit constructor also permits isolated tests without process-wide env changes.
    pub fn new(
        bind: SocketAddr,
        service: HttpService,
        keys: BTreeMap<String, String>,
        allowed_hosts: Vec<String>,
        allowed_origins: Vec<String>,
    ) -> Result<Self> {
        if !bind.ip().is_loopback() && (keys.is_empty() || allowed_hosts.is_empty()) {
            bail!("Non-loopback HTTP binding requires API credentials and SOVEREIGN_ALLOWED_HOSTS");
        }
        let mut credentials: Vec<Credential> = Vec::new();
        for (user_id, key) in keys {
            if user_id.trim().is_empty() || user_id.len() > 128 || key.len() < 32 {
                bail!("HTTP credentials require a nonempty user ID (max 128 bytes) and a key of at least 32 bytes");
            }
            let tag = key_mac(&key).finalize().into_bytes().to_vec();
            if credentials.iter().any(|credential| credential.tag == tag) {
                bail!("Each HTTP principal must have a unique API key");
            }
            credentials.push(Credential {
                principal: Principal { user_id },
                tag,
            });
        }
        let allowed_hosts = if allowed_hosts.is_empty() {
            vec![bind.to_string(), format!("localhost:{}", bind.port())]
        } else {
            allowed_hosts
        };
        let allowed_hosts = allowed_hosts.into_iter().map(|value| {
            let authority = value.parse::<axum::http::uri::Authority>()
                .context("Invalid allowed HTTP authority")?;
            if authority.as_str().contains('@') || authority.as_str().contains('*') {
                bail!("Allowed hosts must be exact authorities without wildcards or user information");
            }
            Ok(authority.as_str().to_ascii_lowercase())
        }).collect::<Result<Vec<_>>>()?;
        for origin in &allowed_origins {
            let uri = origin
                .parse::<axum::http::Uri>()
                .context("Invalid allowed HTTP origin")?;
            if !matches!(uri.scheme_str(), Some("http" | "https"))
                || uri.authority().is_none()
                || uri
                    .path_and_query()
                    .is_some_and(|path| path.as_str() != "/")
                || origin.contains('*')
                || origin.contains('@')
            {
                bail!("Allowed origins must be exact http(s) origins without paths or wildcards");
            }
        }
        Ok(Self {
            service,
            credentials,
            allowed_hosts,
            allowed_origins,
        })
    }

    pub fn requires_auth(&self) -> bool {
        !self.credentials.is_empty()
    }

    pub fn allowed_hosts(&self) -> &[String] {
        &self.allowed_hosts
    }

    pub fn validate_origin_and_host(&self, headers: &HeaderMap) -> Result<(), StatusCode> {
        let host = single_header(headers, "host")?.ok_or(StatusCode::FORBIDDEN)?;
        if !self
            .allowed_hosts
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(host))
        {
            return Err(StatusCode::FORBIDDEN);
        }
        if let Some(origin) = single_header(headers, "origin")? {
            let explicitly_allowed = self.allowed_origins.iter().any(|allowed| allowed == origin);
            // Same-origin browser access is safe only after exact Host validation.
            let same_origin =
                origin == format!("http://{host}") || origin == format!("https://{host}");
            if !explicitly_allowed && !same_origin {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        Ok(())
    }

    pub fn authenticate_token(&self, token: &str) -> Option<Principal> {
        self.credentials.iter().find_map(|credential| {
            key_mac(token)
                .verify_slice(&credential.tag)
                .ok()
                .map(|_| credential.principal.clone())
        })
    }

    pub fn authenticate_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<Option<Principal>, StatusCode> {
        if !self.requires_auth() {
            return Ok(None);
        }
        let token = match self.service {
            HttpService::Proxy => single_header(headers, "x-sovereign-key")?,
            _ => {
                let authorization = single_header(headers, "authorization")?;
                let key = single_header(headers, "x-api-key")?;
                if authorization.is_some() && key.is_some() {
                    return Err(StatusCode::UNAUTHORIZED);
                }
                match authorization {
                    Some(value) => Some(
                        value
                            .strip_prefix("Bearer ")
                            .ok_or(StatusCode::UNAUTHORIZED)?,
                    ),
                    None => key,
                }
            }
        }
        .ok_or(StatusCode::UNAUTHORIZED)?;
        self.authenticate_token(token)
            .map(Some)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

fn read_list(name: &str) -> Result<Vec<String>> {
    match std::env::var(name) {
        Ok(value) => {
            let values = value
                .split(',')
                .map(str::trim)
                .map(String::from)
                .collect::<Vec<_>>();
            if values.iter().any(String::is_empty) {
                bail!("{name} contains an empty entry");
            }
            Ok(values)
        }
        Err(std::env::VarError::NotPresent) => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

fn single_header<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>, StatusCode> {
    let mut values = headers.get_all(name).iter();
    let value = values.next();
    if values.next().is_some() {
        return Err(StatusCode::BAD_REQUEST);
    }
    value
        .map(|value| value.to_str().map_err(|_| StatusCode::BAD_REQUEST))
        .transpose()
}

pub async fn guard(
    State(security): State<Arc<HttpSecurity>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    security.validate_origin_and_host(request.headers())?;
    if let Some(principal) = security.authenticate_headers(request.headers())? {
        request.extensions_mut().insert(principal);
    }
    let (parts, body) = request.into_parts();
    let body = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        to_bytes(body, MAX_REQUEST_BODY_BYTES),
    )
    .await
    .map_err(|_| StatusCode::REQUEST_TIMEOUT)?
    .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    Ok(next.run(Request::from_parts(parts, Body::from(body))).await)
}
