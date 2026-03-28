use chrono::{DateTime, Utc};
use pigeon_macros::Reconstitute;
use uuid::Uuid;

use crate::organization::OrganizationId;
use crate::version::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcConfigId(Uuid);

impl Default for OidcConfigId {
    fn default() -> Self {
        Self::new()
    }
}

impl OidcConfigId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Clone, Reconstitute)]
pub struct OidcConfig {
    id: OidcConfigId,
    org_id: OrganizationId,
    issuer_url: String,
    audience: String,
    jwks_url: String,
    created_at: DateTime<Utc>,
    version: Version,
}

#[derive(Debug, thiserror::Error)]
pub enum OidcConfigError {
    #[error("issuer URL must not be empty")]
    EmptyIssuerUrl,
    #[error("audience must not be empty")]
    EmptyAudience,
    #[error("JWKS URL must not be empty")]
    EmptyJwksUrl,
}

impl OidcConfig {
    pub fn new(
        org_id: OrganizationId,
        issuer_url: String,
        audience: String,
        jwks_url: String,
    ) -> Result<Self, OidcConfigError> {
        if issuer_url.trim().is_empty() {
            return Err(OidcConfigError::EmptyIssuerUrl);
        }
        if audience.trim().is_empty() {
            return Err(OidcConfigError::EmptyAudience);
        }
        if jwks_url.trim().is_empty() {
            return Err(OidcConfigError::EmptyJwksUrl);
        }
        Ok(Self {
            id: OidcConfigId::new(),
            org_id,
            issuer_url,
            audience,
            jwks_url,
            created_at: Utc::now(),
            version: Version::new(0),
        })
    }

    pub fn id(&self) -> &OidcConfigId {
        &self.id
    }

    pub fn org_id(&self) -> &OrganizationId {
        &self.org_id
    }

    pub fn issuer_url(&self) -> &str {
        &self.issuer_url
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }

    pub fn jwks_url(&self) -> &str {
        &self.jwks_url
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::any_oidc_config;

    #[test]
    fn create_oidc_config_with_valid_fields() {
        let config = OidcConfig::new(
            OrganizationId::new(),
            "https://auth.example.com".into(),
            "my-api".into(),
            "https://auth.example.com/.well-known/jwks.json".into(),
        )
        .unwrap();

        assert_eq!(config.issuer_url(), "https://auth.example.com");
        assert_eq!(config.audience(), "my-api");
        assert_eq!(
            config.jwks_url(),
            "https://auth.example.com/.well-known/jwks.json"
        );
    }

    #[test]
    fn reject_empty_issuer_url() {
        let result = OidcConfig::new(
            OrganizationId::new(),
            "".into(),
            "my-api".into(),
            "https://auth.example.com/.well-known/jwks.json".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn reject_whitespace_only_issuer_url() {
        let result = OidcConfig::new(
            OrganizationId::new(),
            "   ".into(),
            "my-api".into(),
            "https://auth.example.com/.well-known/jwks.json".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_audience() {
        let result = OidcConfig::new(
            OrganizationId::new(),
            "https://auth.example.com".into(),
            "".into(),
            "https://auth.example.com/.well-known/jwks.json".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn reject_whitespace_only_audience() {
        let result = OidcConfig::new(
            OrganizationId::new(),
            "https://auth.example.com".into(),
            "   ".into(),
            "https://auth.example.com/.well-known/jwks.json".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_jwks_url() {
        let result = OidcConfig::new(
            OrganizationId::new(),
            "https://auth.example.com".into(),
            "my-api".into(),
            "".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn reject_whitespace_only_jwks_url() {
        let result = OidcConfig::new(
            OrganizationId::new(),
            "https://auth.example.com".into(),
            "my-api".into(),
            "   ".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn new_oidc_config_has_version_zero() {
        let config = any_oidc_config();
        assert_eq!(config.version(), Version::new(0));
    }

    #[test]
    fn oidc_config_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = OidcConfigId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn oidc_config_id_new_is_unique() {
        let id1 = OidcConfigId::new();
        let id2 = OidcConfigId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let state = OidcConfigState::fake();

        let config = OidcConfig::reconstitute(OidcConfigState {
            id: state.id.clone(),
            org_id: state.org_id.clone(),
            issuer_url: state.issuer_url.clone(),
            audience: state.audience.clone(),
            jwks_url: state.jwks_url.clone(),
            created_at: state.created_at,
            version: state.version,
        });

        assert_eq!(*config.id(), state.id);
        assert_eq!(*config.org_id(), state.org_id);
        assert_eq!(config.issuer_url(), state.issuer_url);
        assert_eq!(config.audience(), state.audience);
        assert_eq!(config.jwks_url(), state.jwks_url);
        assert_eq!(*config.created_at(), state.created_at);
        assert_eq!(config.version(), state.version);
    }
}
