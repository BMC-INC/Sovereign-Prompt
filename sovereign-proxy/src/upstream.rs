// HTTP forwarding to upstream LLM providers
use anyhow::Result;
use bytes::Bytes;
use reqwest::{header::HeaderMap, Client};

pub struct UpstreamClient {
    client: Client,
    anthropic_url: String,
    openai_url: String,
}

impl UpstreamClient {
    pub fn new() -> Self {
        let anthropic_url = std::env::var("SOVEREIGN_UPSTREAM_ANTHROPIC")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
        let openai_url = std::env::var("SOVEREIGN_UPSTREAM_OPENAI")
            .unwrap_or_else(|_| "https://api.openai.com".to_string());

        Self {
            client: Client::new(),
            anthropic_url,
            openai_url,
        }
    }

    /// Forward a non-streaming request to the upstream provider.
    /// Returns (status_code, response_headers, response_body_bytes).
    pub async fn forward(
        &self,
        path: &str,
        headers: HeaderMap,
        body: Bytes,
        is_anthropic: bool,
    ) -> Result<(u16, HeaderMap, Bytes)> {
        let base = if is_anthropic {
            &self.anthropic_url
        } else {
            &self.openai_url
        };
        let url = format!("{}{}", base, path);

        let mut req = self.client.post(&url).body(body);

        // Forward relevant headers (auth, content-type, api version, etc.)
        // Skip hop-by-hop headers that shouldn't be forwarded
        for (key, value) in headers.iter() {
            let name = key.as_str().to_lowercase();
            match name.as_str() {
                "host" | "content-length" | "transfer-encoding" | "connection" => continue,
                _ => {
                    req = req.header(key.clone(), value.clone());
                }
            }
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let resp_headers = resp.headers().clone();
        let resp_body = resp.bytes().await?;

        Ok((status, resp_headers, resp_body))
    }

    /// Forward a streaming request, returning the raw reqwest::Response for SSE streaming.
    pub async fn forward_stream(
        &self,
        path: &str,
        headers: HeaderMap,
        body: Bytes,
        is_anthropic: bool,
    ) -> Result<(u16, HeaderMap, reqwest::Response)> {
        let base = if is_anthropic {
            &self.anthropic_url
        } else {
            &self.openai_url
        };
        let url = format!("{}{}", base, path);

        let mut req = self.client.post(&url).body(body);

        for (key, value) in headers.iter() {
            let name = key.as_str().to_lowercase();
            match name.as_str() {
                "host" | "content-length" | "transfer-encoding" | "connection" => continue,
                _ => {
                    req = req.header(key.clone(), value.clone());
                }
            }
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let resp_headers = resp.headers().clone();

        Ok((status, resp_headers, resp))
    }
}
