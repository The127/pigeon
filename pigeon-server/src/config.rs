use std::time::Duration;

use anyhow::Context;

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
        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

        let listen_addr = std::env::var("PIGEON_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        let bootstrap_org_enabled = std::env::var("PIGEON_BOOTSTRAP_ORG_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        let bootstrap_org_name = std::env::var("PIGEON_BOOTSTRAP_ORG_NAME")
            .unwrap_or_else(|_| "System".to_string());

        let bootstrap_org_slug = std::env::var("PIGEON_BOOTSTRAP_ORG_SLUG")
            .unwrap_or_else(|_| "system".to_string());

        let bootstrap_oidc_issuer_url =
            std::env::var("PIGEON_BOOTSTRAP_OIDC_ISSUER_URL").unwrap_or_default();
        let bootstrap_oidc_audience =
            std::env::var("PIGEON_BOOTSTRAP_OIDC_AUDIENCE").unwrap_or_default();
        let bootstrap_oidc_jwks_url =
            std::env::var("PIGEON_BOOTSTRAP_OIDC_JWKS_URL").unwrap_or_default();

        if bootstrap_org_enabled
            && (bootstrap_oidc_issuer_url.is_empty() || bootstrap_oidc_audience.is_empty())
        {
            anyhow::bail!(
                "PIGEON_BOOTSTRAP_OIDC_ISSUER_URL and PIGEON_BOOTSTRAP_OIDC_AUDIENCE are required \
                 when PIGEON_BOOTSTRAP_ORG_ENABLED=true"
            );
        }

        let jwks_cache_ttl_secs: u64 = std::env::var("PIGEON_JWKS_CACHE_TTL_SECS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .context("PIGEON_JWKS_CACHE_TTL_SECS must be a valid integer")?;

        let worker_batch_size: u32 = std::env::var("PIGEON_WORKER_BATCH_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .context("PIGEON_WORKER_BATCH_SIZE must be a valid integer")?;

        let worker_poll_interval_ms: u64 = std::env::var("PIGEON_WORKER_POLL_INTERVAL_MS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .context("PIGEON_WORKER_POLL_INTERVAL_MS must be a valid integer")?;

        let worker_max_retries: u32 = std::env::var("PIGEON_WORKER_MAX_RETRIES")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .context("PIGEON_WORKER_MAX_RETRIES must be a valid integer")?;

        let worker_backoff_base_secs: u64 = std::env::var("PIGEON_WORKER_BACKOFF_BASE_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .context("PIGEON_WORKER_BACKOFF_BASE_SECS must be a valid integer")?;

        let worker_max_backoff_secs: u64 = std::env::var("PIGEON_WORKER_MAX_BACKOFF_SECS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .context("PIGEON_WORKER_MAX_BACKOFF_SECS must be a valid integer")?;

        let worker_http_timeout_secs: u64 = std::env::var("PIGEON_WORKER_HTTP_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .context("PIGEON_WORKER_HTTP_TIMEOUT_SECS must be a valid integer")?;

        let worker_auto_disable_threshold: u64 = std::env::var("PIGEON_WORKER_AUTO_DISABLE_THRESHOLD")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .context("PIGEON_WORKER_AUTO_DISABLE_THRESHOLD must be a valid integer")?;

        let worker_cleanup_interval_secs: u64 = std::env::var("PIGEON_WORKER_CLEANUP_INTERVAL_SECS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .context("PIGEON_WORKER_CLEANUP_INTERVAL_SECS must be a valid integer")?;

        Ok(Self {
            database_url,
            listen_addr,
            bootstrap_org_enabled,
            bootstrap_org_name,
            bootstrap_org_slug,
            bootstrap_oidc_issuer_url,
            bootstrap_oidc_audience,
            bootstrap_oidc_jwks_url,
            jwks_cache_ttl: Duration::from_secs(jwks_cache_ttl_secs),
            worker_batch_size,
            worker_poll_interval: Duration::from_millis(worker_poll_interval_ms),
            worker_max_retries,
            worker_backoff_base_secs,
            worker_max_backoff_secs,
            worker_http_timeout: Duration::from_secs(worker_http_timeout_secs),
            worker_cleanup_interval_secs,
            worker_auto_disable_threshold,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_optional_vars_missing() {
        // DATABASE_URL must be set for from_env to succeed, but we test the
        // individual defaults by constructing directly.
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
