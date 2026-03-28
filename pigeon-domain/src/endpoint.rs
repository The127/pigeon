use chrono::{DateTime, Utc};
use uuid::Uuid;

use pigeon_macros::Reconstitute;

use crate::application::ApplicationId;
use crate::event_type::EventTypeId;
use crate::version::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointId(Uuid);

impl Default for EndpointId {
    fn default() -> Self {
        Self::new()
    }
}

impl EndpointId {
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
pub struct Endpoint {
    id: EndpointId,
    app_id: ApplicationId,
    url: String,
    signing_secret: String,
    enabled: bool,
    event_type_ids: Vec<EventTypeId>,
    created_at: DateTime<Utc>,
    version: Version,
}

#[derive(Debug, thiserror::Error)]
pub enum EndpointError {
    #[error("endpoint URL must not be empty")]
    EmptyUrl,
    #[error("signing secret must not be empty")]
    EmptySigningSecret,
}

impl Endpoint {
    pub fn new(
        app_id: ApplicationId,
        url: String,
        signing_secret: String,
        event_type_ids: Vec<EventTypeId>,
    ) -> Result<Self, EndpointError> {
        if url.trim().is_empty() {
            return Err(EndpointError::EmptyUrl);
        }
        if signing_secret.trim().is_empty() {
            return Err(EndpointError::EmptySigningSecret);
        }

        Ok(Self {
            id: EndpointId::new(),
            app_id,
            url,
            signing_secret,
            enabled: true,
            event_type_ids,
            created_at: Utc::now(),
            version: Version::new(0),
        })
    }

    pub fn id(&self) -> &EndpointId {
        &self.id
    }

    pub fn app_id(&self) -> &ApplicationId {
        &self.app_id
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn signing_secret(&self) -> &str {
        &self.signing_secret
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn event_type_ids(&self) -> &[EventTypeId] {
        &self.event_type_ids
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn update(
        &mut self,
        url: String,
        signing_secret: String,
        event_type_ids: Vec<EventTypeId>,
    ) -> Result<(), EndpointError> {
        if url.trim().is_empty() {
            return Err(EndpointError::EmptyUrl);
        }
        if signing_secret.trim().is_empty() {
            return Err(EndpointError::EmptySigningSecret);
        }
        self.url = url;
        self.signing_secret = signing_secret;
        self.event_type_ids = event_type_ids;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::any_endpoint;

    #[test]
    fn create_endpoint() {
        let ep = Endpoint::new(
            ApplicationId::new(),
            "https://example.com/webhook".into(),
            "whsec_secret123".into(),
            vec![EventTypeId::new()],
        )
        .unwrap();

        assert_eq!(ep.url(), "https://example.com/webhook");
        assert!(ep.enabled());
        assert_eq!(ep.event_type_ids().len(), 1);
    }

    #[test]
    fn reject_empty_url() {
        let result = Endpoint::new(
            ApplicationId::new(),
            "".into(),
            "whsec_secret123".into(),
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_signing_secret() {
        let result = Endpoint::new(
            ApplicationId::new(),
            "https://example.com/webhook".into(),
            "".into(),
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn disable_and_enable() {
        let mut ep = any_endpoint();

        ep.disable();
        assert!(!ep.enabled());

        ep.enable();
        assert!(ep.enabled());
    }

    #[test]
    fn reject_whitespace_only_url() {
        let result = Endpoint::new(
            ApplicationId::new(),
            "   ".into(),
            "whsec_secret123".into(),
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn reject_whitespace_only_signing_secret() {
        let result = Endpoint::new(
            ApplicationId::new(),
            "https://example.com/webhook".into(),
            "   ".into(),
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn new_endpoint_is_enabled_by_default() {
        let ep = any_endpoint();
        assert!(ep.enabled());
    }

    #[test]
    fn new_endpoint_has_version_zero() {
        let ep = any_endpoint();
        assert_eq!(ep.version(), Version::new(0));
    }

    #[test]
    fn endpoint_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = EndpointId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn endpoint_id_new_is_unique() {
        let id1 = EndpointId::new();
        let id2 = EndpointId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let state = EndpointState::fake();

        let ep = Endpoint::reconstitute(EndpointState {
            id: state.id.clone(),
            app_id: state.app_id.clone(),
            url: state.url.clone(),
            signing_secret: state.signing_secret.clone(),
            enabled: state.enabled,
            event_type_ids: state.event_type_ids.clone(),
            created_at: state.created_at,
            version: state.version,
        });

        assert_eq!(*ep.id(), state.id);
        assert_eq!(*ep.app_id(), state.app_id);
        assert_eq!(ep.url(), state.url);
        assert_eq!(ep.signing_secret(), state.signing_secret);
        assert_eq!(ep.enabled(), state.enabled);
        assert_eq!(ep.event_type_ids(), state.event_type_ids);
        assert_eq!(*ep.created_at(), state.created_at);
        assert_eq!(ep.version(), state.version);
    }
}
