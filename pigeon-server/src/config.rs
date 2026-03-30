use std::time::Duration;

use serde::Deserialize;

fn default_listen_addr() -> String {
    "0.0.0.0:3000".to_string()
}
fn default_org_name() -> String {
    "System".to_string()
}
fn default_org_slug() -> String {
    "system".to_string()
}
fn default_jwks_cache_ttl_secs() -> u64 {
    3600
}
fn default_worker_batch_size() -> u32 {
    10
}
fn default_worker_poll_interval_ms() -> u64 {
    1000
}
fn default_worker_max_retries() -> u32 {
    5
}
fn default_worker_backoff_base_secs() -> u64 {
    30
}
fn default_worker_max_backoff_secs() -> u64 {
    3600
}
fn default_worker_http_timeout_secs() -> u64 {
    30
}
fn default_worker_cleanup_interval_secs() -> u64 {
    3600
}
fn default_worker_auto_disable_threshold() -> u64 {
    5
}

/// Raw config deserialized from environment variables.
/// Field names map to env vars via the PIGEON_ prefix (handled by envy).
/// DATABASE_URL is unprefixed (standard convention).
#[derive(Deserialize)]
struct RawConfig {
    // DATABASE_URL is read separately (no PIGEON_ prefix)
    #[serde(default = "default_listen_addr")]
    listen_addr: String,

    #[serde(default)]
    bootstrap_org_enabled: bool,
    #[serde(default = "default_org_name")]
    bootstrap_org_name: String,
    #[serde(default = "default_org_slug")]
    bootstrap_org_slug: String,
    #[serde(default)]
    bootstrap_oidc_issuer_url: String,
    #[serde(default)]
    bootstrap_oidc_audience: String,
    #[serde(default)]
    bootstrap_oidc_jwks_url: String,

    #[serde(default = "default_jwks_cache_ttl_secs")]
    jwks_cache_ttl_secs: u64,

    #[serde(default = "default_worker_batch_size")]
    worker_batch_size: u32,
    #[serde(default = "default_worker_poll_interval_ms")]
    worker_poll_interval_ms: u64,
    #[serde(default = "default_worker_max_retries")]
    worker_max_retries: u32,
    #[serde(default = "default_worker_backoff_base_secs")]
    worker_backoff_base_secs: u64,
    #[serde(default = "default_worker_max_backoff_secs")]
    worker_max_backoff_secs: u64,
    #[serde(default = "default_worker_http_timeout_secs")]
    worker_http_timeout_secs: u64,
    #[serde(default = "default_worker_cleanup_interval_secs")]
    worker_cleanup_interval_secs: u64,
    #[serde(default = "default_worker_auto_disable_threshold")]
    worker_auto_disable_threshold: u64,
}

pub(crate) struct PigeonConfig {
    pub(crate) database_url: String,
    pub(crate) listen_addr: String,
    pub(crate) bootstrap_org_enabled: bool,
    pub(crate) bootstrap_org_name: String,
    pub(crate) bootstrap_org_slug: String,
    pub(crate) bootstrap_oidc_issuer_url: String,
    pub(crate) bootstrap_oidc_audience: String,
    pub(crate) bootstrap_oidc_jwks_url: String,
    pub(crate) jwks_cache_ttl: Duration,
    // Worker
    pub(crate) worker_batch_size: u32,
    pub(crate) worker_poll_interval: Duration,
    pub(crate) worker_max_retries: u32,
    pub(crate) worker_backoff_base_secs: u64,
    pub(crate) worker_max_backoff_secs: u64,
    pub(crate) worker_http_timeout: Duration,
    pub(crate) worker_cleanup_interval_secs: u64,
    pub(crate) worker_auto_disable_threshold: u64,
}

impl PigeonConfig {
    pub(crate) fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable must be set"))?;

        let raw: RawConfig = envy::prefixed("PIGEON_")
            .from_env()
            .map_err(|e| anyhow::anyhow!("Failed to parse configuration: {e}"))?;

        if raw.bootstrap_org_enabled
            && (raw.bootstrap_oidc_issuer_url.is_empty() || raw.bootstrap_oidc_audience.is_empty())
        {
            anyhow::bail!(
                "PIGEON_BOOTSTRAP_OIDC_ISSUER_URL and PIGEON_BOOTSTRAP_OIDC_AUDIENCE are required \
                 when PIGEON_BOOTSTRAP_ORG_ENABLED=true"
            );
        }

        Ok(Self {
            database_url,
            listen_addr: raw.listen_addr,
            bootstrap_org_enabled: raw.bootstrap_org_enabled,
            bootstrap_org_name: raw.bootstrap_org_name,
            bootstrap_org_slug: raw.bootstrap_org_slug,
            bootstrap_oidc_issuer_url: raw.bootstrap_oidc_issuer_url,
            bootstrap_oidc_audience: raw.bootstrap_oidc_audience,
            bootstrap_oidc_jwks_url: raw.bootstrap_oidc_jwks_url,
            jwks_cache_ttl: Duration::from_secs(raw.jwks_cache_ttl_secs),
            worker_batch_size: raw.worker_batch_size,
            worker_poll_interval: Duration::from_millis(raw.worker_poll_interval_ms),
            worker_max_retries: raw.worker_max_retries,
            worker_backoff_base_secs: raw.worker_backoff_base_secs,
            worker_max_backoff_secs: raw.worker_max_backoff_secs,
            worker_http_timeout: Duration::from_secs(raw.worker_http_timeout_secs),
            worker_cleanup_interval_secs: raw.worker_cleanup_interval_secs,
            worker_auto_disable_threshold: raw.worker_auto_disable_threshold,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_optional_vars_missing() {
        let config = PigeonConfig {
            database_url: "postgres://localhost/test".to_string(),
            listen_addr: "0.0.0.0:3000".to_string(),
            bootstrap_org_enabled: false,
            bootstrap_org_name: "System".to_string(),
            bootstrap_org_slug: "system".to_string(),
            bootstrap_oidc_issuer_url: String::new(),
            bootstrap_oidc_audience: String::new(),
            bootstrap_oidc_jwks_url: String::new(),
            jwks_cache_ttl: Duration::from_secs(3600),
            worker_batch_size: 10,
            worker_poll_interval: Duration::from_millis(1000),
            worker_max_retries: 5,
            worker_backoff_base_secs: 30,
            worker_max_backoff_secs: 3600,
            worker_http_timeout: Duration::from_secs(30),
            worker_cleanup_interval_secs: 3600,
            worker_auto_disable_threshold: 5,
        };

        assert!(!config.bootstrap_org_enabled);
        assert_eq!(config.listen_addr, "0.0.0.0:3000");
        assert_eq!(config.bootstrap_org_name, "System");
        assert_eq!(config.bootstrap_org_slug, "system");
        assert_eq!(config.jwks_cache_ttl, Duration::from_secs(3600));
        assert_eq!(config.worker_batch_size, 10);
        assert_eq!(config.worker_poll_interval, Duration::from_millis(1000));
        assert_eq!(config.worker_max_retries, 5);
        assert_eq!(config.worker_backoff_base_secs, 30);
        assert_eq!(config.worker_max_backoff_secs, 3600);
        assert_eq!(config.worker_http_timeout, Duration::from_secs(30));
        assert_eq!(config.worker_cleanup_interval_secs, 3600);
    }
}
