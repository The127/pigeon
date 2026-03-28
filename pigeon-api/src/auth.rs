use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use pigeon_domain::organization::OrganizationId;
use tokio::sync::RwLock;
use tracing::warn;

use crate::state::AppState;

/// Authenticated request context injected into request extensions.
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub org_id: OrganizationId,
    pub user_id: String, // sub claim
}

/// Trait for fetching JWKS, allowing test substitution.
#[async_trait]
pub trait JwksProvider: Send + Sync {
    async fn get_jwks(
        &self,
        jwks_url: &str,
    ) -> Result<jsonwebtoken::jwk::JwkSet, String>;

    /// Force-refresh the JWKS, bypassing any cache.
    async fn refresh_jwks(
        &self,
        jwks_url: &str,
    ) -> Result<jsonwebtoken::jwk::JwkSet, String>;
}

/// Production JWKS provider that fetches via HTTP without caching.
pub struct HttpJwksProvider;

#[async_trait]
impl JwksProvider for HttpJwksProvider {
    async fn get_jwks(
        &self,
        jwks_url: &str,
    ) -> Result<jsonwebtoken::jwk::JwkSet, String> {
        fetch_jwks(jwks_url).await
    }

    async fn refresh_jwks(
        &self,
        jwks_url: &str,
    ) -> Result<jsonwebtoken::jwk::JwkSet, String> {
        fetch_jwks(jwks_url).await
    }
}

/// Caching JWKS provider that wraps HTTP fetches with a TTL-based cache.
///
/// On cache miss or expiry, fetches from the JWKS URL. The `refresh_jwks` method
/// bypasses the cache entirely, which is useful when a JWT contains a `kid` not
/// present in the cached keyset (indicating key rotation at the OIDC provider).
pub struct CachedJwksProvider {
    cache: Arc<RwLock<HashMap<String, CachedEntry>>>,
    ttl: Duration,
}

struct CachedEntry {
    jwks: jsonwebtoken::jwk::JwkSet,
    fetched_at: Instant,
}

impl CachedJwksProvider {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }
}

#[async_trait]
impl JwksProvider for CachedJwksProvider {
    async fn get_jwks(
        &self,
        jwks_url: &str,
    ) -> Result<jsonwebtoken::jwk::JwkSet, String> {
        // Check cache (read lock)
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(jwks_url) {
                if entry.fetched_at.elapsed() < self.ttl {
                    return Ok(entry.jwks.clone());
                }
            }
        }

        // Cache miss or expired -- fetch without holding any lock
        let jwks = fetch_jwks(jwks_url).await?;

        // Update cache (write lock)
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                jwks_url.to_string(),
                CachedEntry {
                    jwks: jwks.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }

        Ok(jwks)
    }

    async fn refresh_jwks(
        &self,
        jwks_url: &str,
    ) -> Result<jsonwebtoken::jwk::JwkSet, String> {
        let jwks = fetch_jwks(jwks_url).await?;

        // Update cache (write lock)
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                jwks_url.to_string(),
                CachedEntry {
                    jwks: jwks.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }

        Ok(jwks)
    }
}

async fn fetch_jwks(jwks_url: &str) -> Result<jsonwebtoken::jwk::JwkSet, String> {
    let response = reqwest::get(jwks_url)
        .await
        .map_err(|e| format!("failed to fetch JWKS: {e}"))?;
    let jwks: jsonwebtoken::jwk::JwkSet = response
        .json()
        .await
        .map_err(|e| format!("failed to parse JWKS: {e}"))?;
    Ok(jwks)
}

/// JWT claims we extract from the token.
#[derive(Debug, serde::Deserialize)]
struct Claims {
    sub: String,
    iss: Option<String>,
    aud: Option<Audience>,
}

/// The `aud` claim can be a single string or an array of strings.
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum Audience {
    Single(String),
    Multiple(Vec<String>),
}

impl Audience {
    #[allow(dead_code)]
    fn contains(&self, value: &str) -> bool {
        match self {
            Audience::Single(s) => s == value,
            Audience::Multiple(v) => v.iter().any(|s| s == value),
        }
    }

    fn first(&self) -> Option<&str> {
        match self {
            Audience::Single(s) => Some(s.as_str()),
            Audience::Multiple(v) => v.first().map(|s| s.as_str()),
        }
    }
}

/// Axum middleware that validates JWT tokens against registered OIDC configs.
///
/// Steps:
/// 1. Extract Bearer token from Authorization header
/// 2. Decode JWT header (unvalidated) to get kid
/// 3. Decode JWT claims (unvalidated) to get iss and aud
/// 4. Look up OidcConfig by (issuer_url, audience) from OidcConfigReadStore
/// 5. If not found -> 401
/// 6. Fetch JWKS from config.jwks_url (cached)
/// 7. Find the key matching kid in JWKS
/// 8. If kid not found, force-refresh JWKS once (key rotation) and retry
/// 9. Validate JWT (signature, exp, iss, aud)
/// 10. Extract sub claim
/// 11. Create AuthContext { org_id: config.org_id, user_id: sub }
/// 12. Insert AuthContext into request extensions
/// 13. Call next
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    // 1. Extract Bearer token
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization header format"))?;

    // 2. Decode JWT header to get kid
    let header = decode_header(token)
        .map_err(|e| {
            warn!("Failed to decode JWT header: {e}");
            (StatusCode::UNAUTHORIZED, "Invalid token header")
        })?;

    let kid = header
        .kid
        .as_deref()
        .ok_or((StatusCode::UNAUTHORIZED, "Token missing kid header"))?;

    // 3. Decode claims without validation to get iss and aud
    let mut no_validate = Validation::new(Algorithm::RS256);
    no_validate.insecure_disable_signature_validation();
    no_validate.validate_exp = false;
    no_validate.validate_aud = false;
    no_validate.set_required_spec_claims::<String>(&[]);

    let unvalidated = decode::<Claims>(
        token,
        &DecodingKey::from_secret(b"dummy"),
        &no_validate,
    )
    .map_err(|e| {
        warn!("Failed to decode JWT claims: {e}");
        (StatusCode::UNAUTHORIZED, "Invalid token claims")
    })?;

    let issuer = unvalidated
        .claims
        .iss
        .as_deref()
        .ok_or((StatusCode::UNAUTHORIZED, "Token missing iss claim"))?;

    let audience_claim = unvalidated
        .claims
        .aud
        .as_ref()
        .ok_or((StatusCode::UNAUTHORIZED, "Token missing aud claim"))?;

    let audience = audience_claim
        .first()
        .ok_or((StatusCode::UNAUTHORIZED, "Token has empty aud claim"))?;

    // 4. Look up OidcConfig
    let config = state
        .oidc_config_read_store
        .find_by_issuer_and_audience(issuer, audience)
        .await
        .map_err(|e| {
            warn!("Failed to look up OIDC config: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal error during auth")
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Unknown issuer/audience"))?;

    // 5. Fetch JWKS (cached)
    let jwks = state
        .jwks_provider
        .get_jwks(config.jwks_url())
        .await
        .map_err(|e| {
            warn!("Failed to fetch JWKS: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch JWKS")
        })?;

    // 6. Find matching key by kid; on miss, force-refresh once (key rotation)
    let jwk = match jwks.keys.iter().find(|k| k.common.key_id.as_deref() == Some(kid)) {
        Some(jwk) => jwk.clone(),
        None => {
            let refreshed = state
                .jwks_provider
                .refresh_jwks(config.jwks_url())
                .await
                .map_err(|e| {
                    warn!("Failed to refresh JWKS: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to refresh JWKS")
                })?;

            refreshed
                .keys
                .into_iter()
                .find(|k| k.common.key_id.as_deref() == Some(kid))
                .ok_or((StatusCode::UNAUTHORIZED, "Key ID not found in JWKS"))?
        }
    };

    let decoding_key = DecodingKey::from_jwk(&jwk).map_err(|e| {
        warn!("Failed to create decoding key from JWK: {e}");
        (StatusCode::UNAUTHORIZED, "Invalid JWK")
    })?;

    // 7. Validate JWT
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);

    let validated = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        warn!("JWT validation failed: {e}");
        (StatusCode::UNAUTHORIZED, "Token validation failed")
    })?;

    // 8. Build AuthContext
    let auth_context = AuthContext {
        org_id: config.org_id().clone(),
        user_id: validated.claims.sub,
    };

    // 9. Insert into extensions and proceed
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// Wraps a FakeCountingJwksProvider inside a CachedJwksProvider-like structure
    /// to test caching logic in isolation.
    struct TestableCache {
        cache: Arc<RwLock<HashMap<String, CachedEntry>>>,
        ttl: Duration,
        fetch_count: Arc<AtomicUsize>,
        jwks: jsonwebtoken::jwk::JwkSet,
    }

    impl TestableCache {
        fn new(ttl: Duration) -> Self {
            Self {
                cache: Arc::new(RwLock::new(HashMap::new())),
                ttl,
                fetch_count: Arc::new(AtomicUsize::new(0)),
                jwks: jsonwebtoken::jwk::JwkSet { keys: vec![] },
            }
        }

        async fn get(&self, url: &str) -> jsonwebtoken::jwk::JwkSet {
            {
                let cache = self.cache.read().await;
                if let Some(entry) = cache.get(url) {
                    if entry.fetched_at.elapsed() < self.ttl {
                        return entry.jwks.clone();
                    }
                }
            }

            self.fetch_count.fetch_add(1, Ordering::SeqCst);

            {
                let mut cache = self.cache.write().await;
                cache.insert(
                    url.to_string(),
                    CachedEntry {
                        jwks: self.jwks.clone(),
                        fetched_at: Instant::now(),
                    },
                );
            }

            self.jwks.clone()
        }

        async fn refresh(&self, url: &str) -> jsonwebtoken::jwk::JwkSet {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);

            {
                let mut cache = self.cache.write().await;
                cache.insert(
                    url.to_string(),
                    CachedEntry {
                        jwks: self.jwks.clone(),
                        fetched_at: Instant::now(),
                    },
                );
            }

            self.jwks.clone()
        }

        fn fetch_count(&self) -> usize {
            self.fetch_count.load(Ordering::SeqCst)
        }
    }

    #[tokio::test]
    async fn first_call_fetches_second_returns_cached() {
        let cache = TestableCache::new(Duration::from_secs(3600));

        let _jwks1 = cache.get("https://example.com/.well-known/jwks.json").await;
        assert_eq!(cache.fetch_count(), 1);

        let _jwks2 = cache.get("https://example.com/.well-known/jwks.json").await;
        assert_eq!(cache.fetch_count(), 1); // No additional fetch
    }

    #[tokio::test]
    async fn expired_entry_triggers_refetch() {
        let cache = TestableCache::new(Duration::from_millis(0)); // immediate expiry

        let _jwks1 = cache.get("https://example.com/.well-known/jwks.json").await;
        assert_eq!(cache.fetch_count(), 1);

        let _jwks2 = cache.get("https://example.com/.well-known/jwks.json").await;
        assert_eq!(cache.fetch_count(), 2); // Expired, so re-fetched
    }

    #[tokio::test]
    async fn force_refresh_bypasses_cache() {
        let cache = TestableCache::new(Duration::from_secs(3600));

        let _jwks1 = cache.get("https://example.com/.well-known/jwks.json").await;
        assert_eq!(cache.fetch_count(), 1);

        // Force refresh should fetch again even though cache is still valid
        let _jwks2 = cache.refresh("https://example.com/.well-known/jwks.json").await;
        assert_eq!(cache.fetch_count(), 2);
    }
}
