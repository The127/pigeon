use fake::faker::name::en::Name;
use fake::Fake;
use uuid::Uuid;

use crate::application::{Application, ApplicationId, ApplicationState};
use crate::attempt::{Attempt, AttemptId, AttemptState, AttemptStatus};
use crate::dead_letter::{DeadLetter, DeadLetterId, DeadLetterState};
use crate::endpoint::{Endpoint, EndpointId, EndpointState};
use crate::event_type::{EventType, EventTypeId, EventTypeState};
use crate::message::{IdempotencyKey, Message, MessageId, MessageState};
use crate::oidc_config::{OidcConfig, OidcConfigId, OidcConfigState};
use crate::organization::{Organization, OrganizationId, OrganizationState};
use crate::version::Version;

// --- State bag builders (random data, all fields populated) ---

impl ApplicationState {
    pub fn fake() -> Self {
        let name: String = Name().fake();
        Self {
            id: ApplicationId::new(),
            org_id: OrganizationId::new(),
            name,
            uid: format!("app_{}", Uuid::new_v4()),
            created_at: chrono::Utc::now(),
            version: Version::new(0),
        }
    }
}

impl EventTypeState {
    pub fn fake() -> Self {
        Self {
            id: EventTypeId::new(),
            app_id: ApplicationId::new(),
            name: format!("event.{}", Uuid::new_v4()),
            schema: None,
            system: false,
            created_at: chrono::Utc::now(),
            version: Version::new(0),
        }
    }
}

impl EndpointState {
    pub fn fake() -> Self {
        Self {
            id: EndpointId::new(),
            app_id: ApplicationId::new(),
            name: crate::name_generator::generate_name(),
            url: format!("https://example.com/webhook/{}", Uuid::new_v4()),
            signing_secrets: vec![crate::endpoint::generate_signing_secret()],
            enabled: true,
            event_type_ids: vec![EventTypeId::new()],
            created_at: chrono::Utc::now(),
            version: Version::new(0),
        }
    }
}

impl MessageState {
    pub fn fake() -> Self {
        Self {
            id: MessageId::new(),
            app_id: ApplicationId::new(),
            event_type_id: EventTypeId::new(),
            payload: serde_json::json!({"fake": true}),
            idempotency_key: IdempotencyKey::generate(),
            idempotency_expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            created_at: chrono::Utc::now(),
            version: Version::new(0),
        }
    }
}

impl AttemptState {
    pub fn fake() -> Self {
        Self {
            id: AttemptId::new(),
            message_id: MessageId::new(),
            endpoint_id: EndpointId::new(),
            status: AttemptStatus::Pending,
            response_code: None,
            response_body: None,
            attempted_at: None,
            next_attempt_at: Some(chrono::Utc::now()),
            attempt_number: 1,
            duration_ms: None,
            version: Version::new(0),
        }
    }
}

impl DeadLetterState {
    pub fn fake() -> Self {
        Self {
            id: DeadLetterId::new(),
            message_id: MessageId::new(),
            endpoint_id: EndpointId::new(),
            app_id: ApplicationId::new(),
            last_response_code: Some(500),
            last_response_body: Some("Internal Server Error".into()),
            dead_lettered_at: chrono::Utc::now(),
            replayed_at: None,
            version: Version::new(0),
        }
    }
}

impl OrganizationState {
    pub fn fake() -> Self {
        let name: String = Name().fake();
        let slug = format!("org-{}", Uuid::new_v4().simple());
        Self {
            id: OrganizationId::new(),
            name,
            slug,
            created_at: chrono::Utc::now(),
            version: Version::new(0),
        }
    }
}

impl OidcConfigState {
    pub fn fake() -> Self {
        Self {
            id: OidcConfigId::new(),
            org_id: OrganizationId::new(),
            issuer_url: format!("https://auth-{}.example.com", Uuid::new_v4().simple()),
            audience: format!("api-{}", Uuid::new_v4().simple()),
            jwks_url: format!(
                "https://auth-{}.example.com/.well-known/jwks.json",
                Uuid::new_v4().simple()
            ),
            created_at: chrono::Utc::now(),
            version: Version::new(0),
        }
    }
}

// --- Entity factory functions (go through constructors, always valid) ---

pub fn any_organization() -> Organization {
    let name: String = Name().fake();
    let slug = format!("org-{}", Uuid::new_v4().simple());
    Organization::new(name, slug).unwrap()
}

pub fn any_application() -> Application {
    let name: String = Name().fake();
    Application::new(OrganizationId::new(), name, format!("app_{}", Uuid::new_v4())).unwrap()
}

pub fn any_event_type() -> EventType {
    EventType::new(
        ApplicationId::new(),
        format!("event.{}", Uuid::new_v4()),
        None,
    )
    .unwrap()
}

pub fn any_endpoint() -> Endpoint {
    Endpoint::new(
        ApplicationId::new(),
        None,
        format!("https://example.com/webhook/{}", Uuid::new_v4()),
        vec![EventTypeId::new()],
    )
    .unwrap()
}

pub fn any_message() -> Message {
    Message::new(
        ApplicationId::new(),
        EventTypeId::new(),
        serde_json::json!({"fake": true}),
        None,
        chrono::Duration::hours(24),
    )
    .unwrap()
}

pub fn any_attempt() -> Attempt {
    Attempt::new(MessageId::new(), EndpointId::new(), chrono::Utc::now())
}

pub fn any_dead_letter() -> DeadLetter {
    DeadLetter::new(
        MessageId::new(),
        EndpointId::new(),
        ApplicationId::new(),
        Some(500),
        Some("error".into()),
    )
}

pub fn any_oidc_config() -> OidcConfig {
    OidcConfig::new(
        OrganizationId::new(),
        format!("https://auth-{}.example.com", Uuid::new_v4().simple()),
        format!("api-{}", Uuid::new_v4().simple()),
        format!(
            "https://auth-{}.example.com/.well-known/jwks.json",
            Uuid::new_v4().simple()
        ),
    )
    .unwrap()
}
