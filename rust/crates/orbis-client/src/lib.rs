//! `orbis-client` — thin, testable client for TomTom Orbis APIs.
//!
//! Security posture:
//! - The API key is **never** stored in the client struct's `Debug` output and
//!   is read from an environment secret only at construction time.
//! - Base URLs are fixed to documented TomTom hosts; no caller-controlled URLs
//!   (prevents SSRF / key exfiltration through config mistakes).
//! - Orbis has a different data model/API surface from legacy TomTom APIs, so
//!   this crate is deliberately separate from any legacy clients.

use serde::Deserialize;
use std::fmt;

pub const MAP_DISPLAY_BASE: &str = "https://api.tomtom.com/maps-sdk-for-web/call";
pub const ROUTING_BASE: &str = "https://api.tomtom.com/routing/1";

#[derive(Debug, thiserror::Error)]
pub enum OrbisError {
    #[error("missing API key: set TOMTOM_API_KEY at runtime (never commit keys)")]
    MissingApiKey,
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unexpected status {status} from {url}")]
    UnexpectedStatus { status: u16, url: String },
}

/// Redacting wrapper so `{key:?}` never leaks into logs or panics.
#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn from_env() -> Result<Self, OrbisError> {
        std::env::var("TOMTOM_API_KEY")
            .map(Self)
            .map_err(|_| OrbisError::MissingApiKey)
    }

    #[cfg(test)]
    pub fn for_tests() -> Self {
        Self("test-key-must-never-be-real".into())
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

pub struct OrbisClient {
    http: reqwest::Client,
    key: ApiKey,
    user_agent: String,
}

/// Metadata returned by the Map Display product-information endpoint.
#[derive(Debug, Deserialize)]
pub struct ProductInfo {
    pub brand: Option<String>,
    #[serde(rename = "mapType")]
    pub map_type: Option<String>,
    #[serde(rename = "version")]
    pub version: Option<String>,
}

impl OrbisClient {
    pub fn new(key: ApiKey) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("default TLS client must build"),
            key,
            user_agent: concat!("orbis-enhancement/", env!("CARGO_PKG_VERSION")).to_string(),
        }
    }

    /// URL for one vector tile of an Orbis Map Display layer.
    /// Format per public-preview docs:
    /// `{base}/{layer}/{style}/{version}/{z}/{y}/{x}.pbf?key=...`
    pub fn tile_url(&self, layer: &str, style: &str, version: &str, z: u8, x: u64, y: u64) -> String {
        format!(
            "{}/OrbisMap/{}:{}/{}/{}/{}/{}.pbf?key={}",
            MAP_DISPLAY_BASE, layer, style, version, z, y, x, self.key.0
        )
    }

    /// Fetch a tile. Kept async; callers batch with tokio.
    pub async fn fetch_tile(&self, layer: &str, style: &str, version: &str, z: u8, x: u64, y: u64) -> Result<Vec<u8>, OrbisError> {
        let url = self.tile_url(layer, style, version, z, x, y);
        let resp = self
            .http
            .get(&url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(OrbisError::UnexpectedStatus { status: status.as_u16(), url: redact(&url) });
        }
        Ok(resp.bytes().await?.to_vec())
    }

    pub async fn product_info(&self, layer: &str, style: &str, version: &str) -> Result<ProductInfo, OrbisError> {
        let url = format!(
            "{}/OrbisMap/{}:{}/{}/productInformation?key={}",
            MAP_DISPLAY_BASE, layer, style, version, self.key.0
        );
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(OrbisError::UnexpectedStatus { status: status.as_u16(), url: redact(&url) });
        }
        Ok(resp.json().await?)
    }
}

/// Strip the query string so error messages/logs can never carry the key.
fn redact(url: &str) -> String {
    match url.split_once('?') {
        Some((base, _)) => format!("{base}?key=***"),
        None => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_redacts_key() {
        let c = OrbisClient::new(ApiKey::for_tests());
        let dbg = format!("{:?}", c.key);
        assert_eq!(dbg, "ApiKey(***)");
    }

    #[test]
    fn tile_url_shape_matches_docs() {
        let c = OrbisClient::new(ApiKey::for_tests());
        let u = c.tile_url("base", "web", "v1", 12, 2045, 1215);
        assert!(u.starts_with(MAP_DISPLAY_BASE));
        assert!(u.contains("/12/1215/2045.pbf")); // z/y/x order
        assert!(u.contains("base:web"));
    }

    #[test]
    fn error_urls_are_redacted() {
        let r = redact("https://api.tomtom.com/x/y.pbf?key=SECRET123");
        assert!(!r.contains("SECRET123"));
        assert!(r.ends_with("key=***"));
    }

    #[test]
    fn missing_key_fails_construction_from_env() {
        // Ensure the env var is absent for this test process namespace.
        std::env::remove_var("TOMTOM_API_KEY");
        assert!(matches!(ApiKey::from_env(), Err(OrbisError::MissingApiKey)));
    }
}
