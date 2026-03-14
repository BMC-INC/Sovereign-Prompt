use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct CryptoEngine {
    hmac_key: Vec<u8>,
}

impl CryptoEngine {
    pub fn new(hmac_key: &[u8]) -> Self {
        Self {
            hmac_key: hmac_key.to_vec(),
        }
    }

    /// SHA-256 hash of content, returned as lowercase hex string.
    pub fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Compute content_hash for a PromptRecord: SHA-256(original || "||" || refined).
    pub fn compute_content_hash(original: &str, refined: &str) -> String {
        let combined = format!("{}||{}", original, refined);
        Self::hash_content(&combined)
    }

    /// Compute output_hash: SHA-256(output_text).
    pub fn compute_output_hash(output: &str) -> String {
        Self::hash_content(output)
    }

    /// HMAC-SHA256 sign over content_hash, returns hex.
    pub fn sign(&self, content_hash: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(&self.hmac_key).expect("HMAC accepts any key length");
        mac.update(content_hash.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Verify HMAC signature against content_hash.
    pub fn verify(&self, content_hash: &str, signature: &str) -> bool {
        let mut mac =
            HmacSha256::new_from_slice(&self.hmac_key).expect("HMAC accepts any key length");
        mac.update(content_hash.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());
        expected == signature
    }

    /// Verify hash chain: content_hash was computed from original+refined,
    /// and output_hash was computed from output.
    pub fn verify_hash_chain(
        original: &str,
        refined: &str,
        expected_content_hash: &str,
        output: Option<&str>,
        expected_output_hash: Option<&str>,
    ) -> bool {
        let content_ok = Self::compute_content_hash(original, refined) == expected_content_hash;
        let output_ok = match (output, expected_output_hash) {
            (Some(o), Some(eh)) => Self::compute_output_hash(o) == eh,
            (None, None) => true,
            _ => false,
        };
        content_ok && output_ok
    }
}
