use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::attempt::Attempt;
use pigeon_domain::dead_letter::{DeadLetter, DeadLetterId};
use pigeon_domain::endpoint::{Endpoint, EndpointId};
use pigeon_domain::event_type::{EventType, EventTypeId};
use pigeon_domain::message::{Message, MessageId};

use crate::ports::message_status::MessageWithStatus;
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId};
use pigeon_domain::organization::{Organization, OrganizationId};

use crate::error::ApplicationError;

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait ApplicationStore: Send + Sync {
    async fn insert(&mut self, application: &Application) -> Result<(), ApplicationError>;
    async fn find_by_id(
        &self,
        id: &ApplicationId,
    ) -> Result<Option<Application>, ApplicationError>;
    async fn save(&mut self, application: &Application) -> Result<(), ApplicationError>;
    async fn delete(&mut self, id: &ApplicationId) -> Result<(), ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait ApplicationReadStore: Send + Sync {
    async fn find_by_id(
        &self,
        id: &ApplicationId,
    ) -> Result<Option<Application>, ApplicationError>;
    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
        search: Option<String>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Application>, ApplicationError>;
    async fn count_by_org(
        &self,
        org_id: &OrganizationId,
        search: Option<String>,
    ) -> Result<u64, ApplicationError>;
}

pub struct InsertMessageResult {
    pub message: Message,
    pub was_existing: bool,
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait MessageStore: Send + Sync {
    /// Inserts the message, or returns the existing one if the idempotency key
    /// matches an unexpired entry for the same app_id.
    /// The org_id is used to verify the app belongs to the caller's org via SQL JOIN.
    async fn insert_or_get_existing(
        &mut self,
        message: &Message,
        org_id: &OrganizationId,
    ) -> Result<InsertMessageResult, ApplicationError>;

    /// Deletes messages whose idempotency keys have expired before `now`.
    /// Returns the number of expired keys removed.
    async fn expire_idempotency_keys(&self, now: DateTime<Utc>) -> Result<u64, ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait AttemptStore: Send + Sync {
    async fn insert(&mut self, attempt: &Attempt) -> Result<(), ApplicationError>;
    async fn find_by_id(
        &self,
        id: &pigeon_domain::attempt::AttemptId,
        org_id: &OrganizationId,
    ) -> Result<Option<Attempt>, ApplicationError>;
    async fn save(&mut self, attempt: &Attempt) -> Result<(), ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait EndpointStore: Send + Sync {
    async fn insert(&mut self, endpoint: &Endpoint) -> Result<(), ApplicationError>;
    async fn find_by_id(
        &self,
        id: &EndpointId,
        org_id: &OrganizationId,
    ) -> Result<Option<Endpoint>, ApplicationError>;
    async fn find_by_app_and_id(
        &self,
        id: &EndpointId,
        app_id: &ApplicationId,
    ) -> Result<Option<Endpoint>, ApplicationError>;
    async fn save(&mut self, endpoint: &Endpoint) -> Result<(), ApplicationError>;
    async fn delete(&mut self, id: &EndpointId) -> Result<(), ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait EndpointReadStore: Send + Sync {
    async fn find_enabled_by_app_and_event_type(
        &self,
        app_id: &ApplicationId,
        event_type_id: &EventTypeId,
        org_id: &OrganizationId,
    ) -> Result<Vec<Endpoint>, ApplicationError>;
    async fn find_by_id(
        &self,
        id: &EndpointId,
        org_id: &OrganizationId,
    ) -> Result<Option<Endpoint>, ApplicationError>;
    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Endpoint>, ApplicationError>;
    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait EventTypeStore: Send + Sync {
    async fn insert(&mut self, event_type: &EventType) -> Result<(), ApplicationError>;
    async fn find_by_id(
        &self,
        id: &EventTypeId,
        org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError>;
    async fn save(&mut self, event_type: &EventType) -> Result<(), ApplicationError>;
    async fn delete(&mut self, id: &EventTypeId) -> Result<(), ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait EventTypeReadStore: Send + Sync {
    async fn find_by_id(
        &self,
        id: &EventTypeId,
        org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError>;
    async fn find_by_app_and_name(
        &self,
        app_id: &ApplicationId,
        name: &str,
        org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError>;
    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<EventType>, ApplicationError>;
    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait DeadLetterStore: Send + Sync {
    async fn insert(&mut self, dead_letter: &DeadLetter) -> Result<(), ApplicationError>;
    async fn find_by_id(
        &self,
        id: &DeadLetterId,
        org_id: &OrganizationId,
    ) -> Result<Option<DeadLetter>, ApplicationError>;
    async fn save(&mut self, dead_letter: &DeadLetter) -> Result<(), ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait DeadLetterReadStore: Send + Sync {
    /// Count dead letters for an endpoint since the last successful delivery.
    async fn consecutive_failure_count(
        &self,
        endpoint_id: &EndpointId,
    ) -> Result<u64, ApplicationError>;
    async fn find_by_id(
        &self,
        id: &DeadLetterId,
        org_id: &OrganizationId,
    ) -> Result<Option<DeadLetter>, ApplicationError>;
    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        endpoint_id: Option<EndpointId>,
        replayed: Option<bool>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<DeadLetter>, ApplicationError>;
    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        endpoint_id: Option<EndpointId>,
        replayed: Option<bool>,
    ) -> Result<u64, ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait MessageReadStore: Send + Sync {
    async fn find_by_id(
        &self,
        id: &MessageId,
        org_id: &OrganizationId,
    ) -> Result<Option<MessageWithStatus>, ApplicationError>;
    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        event_type_id: Option<EventTypeId>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<MessageWithStatus>, ApplicationError>;
    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        event_type_id: Option<EventTypeId>,
    ) -> Result<u64, ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait AttemptReadStore: Send + Sync {
    async fn list_by_message(
        &self,
        message_id: &MessageId,
        org_id: &OrganizationId,
    ) -> Result<Vec<Attempt>, ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait OrganizationStore: Send + Sync {
    async fn insert(&mut self, organization: &Organization) -> Result<(), ApplicationError>;
    async fn find_by_id(
        &self,
        id: &OrganizationId,
    ) -> Result<Option<Organization>, ApplicationError>;
    async fn save(&mut self, organization: &Organization) -> Result<(), ApplicationError>;
    async fn delete(&mut self, id: &OrganizationId) -> Result<(), ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait OrganizationReadStore: Send + Sync {
    async fn find_by_id(
        &self,
        id: &OrganizationId,
    ) -> Result<Option<Organization>, ApplicationError>;
    async fn find_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<Organization>, ApplicationError>;
    async fn list(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Organization>, ApplicationError>;
    async fn count(&self) -> Result<u64, ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait OidcConfigStore: Send + Sync {
    async fn insert(&mut self, config: &OidcConfig) -> Result<(), ApplicationError>;
    async fn find_by_id(
        &self,
        id: &OidcConfigId,
    ) -> Result<Option<OidcConfig>, ApplicationError>;
    async fn count_by_org(&self, org_id: &OrganizationId) -> Result<u64, ApplicationError>;
    async fn delete(&mut self, id: &OidcConfigId) -> Result<(), ApplicationError>;
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait OidcConfigReadStore: Send + Sync {
    async fn find_by_issuer_and_audience(
        &self,
        issuer_url: &str,
        audience: &str,
    ) -> Result<Option<OidcConfig>, ApplicationError>;
    async fn find_by_id(
        &self,
        id: &OidcConfigId,
    ) -> Result<Option<OidcConfig>, ApplicationError>;
    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<OidcConfig>, ApplicationError>;
    async fn count_by_org(
        &self,
        org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError>;
}
