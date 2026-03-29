use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::attempt::Attempt;
use pigeon_domain::dead_letter::{DeadLetter, DeadLetterId};
use pigeon_domain::endpoint::Endpoint;
use pigeon_domain::event_type::{EventType, EventTypeId};
use pigeon_domain::message::Message;
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId};
use pigeon_domain::organization::{Organization, OrganizationId};

use crate::error::ApplicationError;
use crate::ports::audit_store::{AuditEntry, AuditStore};
use pigeon_domain::endpoint::EndpointId;

use crate::ports::stores::{
    ApplicationReadStore, ApplicationStore, AttemptStore, DeadLetterStore, EndpointReadStore,
    EndpointStore, EventTypeReadStore, EventTypeStore, InsertMessageResult, MessageStore,
    OidcConfigStore, OrganizationReadStore, OrganizationStore,
};
use crate::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};

/// Tracks operations performed on the fake UoW for test assertions.
#[derive(Debug, Clone, Default)]
pub struct OperationLog {
    pub entries: Arc<Mutex<Vec<String>>>,
}

impl OperationLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, op: &str) {
        self.entries.lock().unwrap().push(op.to_string());
    }

    pub fn entries(&self) -> Vec<String> {
        self.entries.lock().unwrap().clone()
    }
}

// --- Shared application data across UoW instances ---

#[derive(Debug, Clone, Default)]
pub struct SharedApplicationData {
    pub applications: Arc<Mutex<Vec<Application>>>,
}

// --- Shared event type data across UoW instances ---

#[derive(Debug, Clone, Default)]
pub struct SharedEventTypeData {
    pub event_types: Arc<Mutex<Vec<EventType>>>,
}

// --- Shared endpoint data across UoW instances ---

#[derive(Debug, Clone, Default)]
pub struct SharedEndpointData {
    pub endpoints: Arc<Mutex<Vec<Endpoint>>>,
}

// --- Shared message data across UoW instances ---

#[derive(Debug, Clone, Default)]
pub struct SharedMessageData {
    pub messages: Arc<Mutex<Vec<Message>>>,
}

// --- Shared organization data across UoW instances ---

#[derive(Debug, Clone, Default)]
pub struct SharedOrganizationData {
    pub organizations: Arc<Mutex<Vec<Organization>>>,
}

// --- Shared OIDC config data across UoW instances ---

#[derive(Debug, Clone, Default)]
pub struct SharedOidcConfigData {
    pub oidc_configs: Arc<Mutex<Vec<OidcConfig>>>,
}

// --- Fake Stores ---

pub struct FakeApplicationStore {
    log: OperationLog,
    data: SharedApplicationData,
}

impl FakeApplicationStore {
    pub(crate) fn new(log: OperationLog, data: SharedApplicationData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl ApplicationStore for FakeApplicationStore {
    async fn insert(&mut self, application: &Application) -> Result<(), ApplicationError> {
        self.log.record("application_store:insert");
        self.data
            .applications
            .lock()
            .unwrap()
            .push(application.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &ApplicationId,
    ) -> Result<Option<Application>, ApplicationError> {
        self.log.record("application_store:find_by_id");
        let apps = self.data.applications.lock().unwrap();
        Ok(apps.iter().find(|a| a.id() == id).cloned())
    }

    async fn save(&mut self, application: &Application) -> Result<(), ApplicationError> {
        self.log.record("application_store:save");
        let mut apps = self.data.applications.lock().unwrap();
        if let Some(existing) = apps.iter_mut().find(|a| a.id() == application.id()) {
            *existing = application.clone();
        }
        Ok(())
    }

    async fn delete(&mut self, id: &ApplicationId) -> Result<(), ApplicationError> {
        self.log.record("application_store:delete");
        let mut apps = self.data.applications.lock().unwrap();
        apps.retain(|a| a.id() != id);
        Ok(())
    }
}

// --- Fake ApplicationReadStore ---

pub struct FakeApplicationReadStore {
    log: OperationLog,
    data: SharedApplicationData,
}

impl FakeApplicationReadStore {
    pub fn new(log: OperationLog, data: SharedApplicationData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl ApplicationReadStore for FakeApplicationReadStore {
    async fn find_by_id(
        &self,
        id: &ApplicationId,
    ) -> Result<Option<Application>, ApplicationError> {
        self.log.record("application_read_store:find_by_id");
        let apps = self.data.applications.lock().unwrap();
        Ok(apps.iter().find(|a| a.id() == id).cloned())
    }

    async fn list_by_org(
        &self,
        org_id: &pigeon_domain::organization::OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Application>, ApplicationError> {
        self.log.record("application_read_store:list_by_org");
        let apps = self.data.applications.lock().unwrap();
        let result = apps
            .iter()
            .filter(|a| a.org_id() == org_id)
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn count_by_org(
        &self,
        org_id: &pigeon_domain::organization::OrganizationId,
    ) -> Result<u64, ApplicationError> {
        self.log.record("application_read_store:count_by_org");
        let apps = self.data.applications.lock().unwrap();
        Ok(apps.iter().filter(|a| a.org_id() == org_id).count() as u64)
    }
}

// --- Fake EventTypeStore ---

pub struct FakeEventTypeStore {
    log: OperationLog,
    data: SharedEventTypeData,
}

impl FakeEventTypeStore {
    pub(crate) fn new(log: OperationLog, data: SharedEventTypeData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl EventTypeStore for FakeEventTypeStore {
    async fn insert(&mut self, event_type: &EventType) -> Result<(), ApplicationError> {
        self.log.record("event_type_store:insert");
        self.data
            .event_types
            .lock()
            .unwrap()
            .push(event_type.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &EventTypeId,
        _org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError> {
        self.log.record("event_type_store:find_by_id");
        let ets = self.data.event_types.lock().unwrap();
        Ok(ets.iter().find(|et| et.id() == id).cloned())
    }

    async fn save(&mut self, event_type: &EventType) -> Result<(), ApplicationError> {
        self.log.record("event_type_store:save");
        let mut ets = self.data.event_types.lock().unwrap();
        if let Some(existing) = ets.iter_mut().find(|et| et.id() == event_type.id()) {
            *existing = event_type.clone();
        }
        Ok(())
    }

    async fn delete(&mut self, id: &EventTypeId) -> Result<(), ApplicationError> {
        self.log.record("event_type_store:delete");
        let mut ets = self.data.event_types.lock().unwrap();
        ets.retain(|et| et.id() != id);
        Ok(())
    }
}

// --- Fake EventTypeReadStore ---

pub struct FakeEventTypeReadStore {
    log: OperationLog,
    data: SharedEventTypeData,
}

impl FakeEventTypeReadStore {
    pub fn new(log: OperationLog, data: SharedEventTypeData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl EventTypeReadStore for FakeEventTypeReadStore {
    async fn find_by_id(
        &self,
        id: &EventTypeId,
        _org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError> {
        self.log.record("event_type_read_store:find_by_id");
        let ets = self.data.event_types.lock().unwrap();
        Ok(ets.iter().find(|et| et.id() == id).cloned())
    }

    async fn find_by_app_and_name(
        &self,
        app_id: &ApplicationId,
        name: &str,
        _org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError> {
        self.log
            .record("event_type_read_store:find_by_app_and_name");
        let ets = self.data.event_types.lock().unwrap();
        Ok(ets
            .iter()
            .find(|et| et.app_id() == app_id && et.name() == name)
            .cloned())
    }

    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        _org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<EventType>, ApplicationError> {
        self.log.record("event_type_read_store:list_by_app");
        let ets = self.data.event_types.lock().unwrap();
        let result = ets
            .iter()
            .filter(|et| et.app_id() == app_id)
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        _org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        self.log.record("event_type_read_store:count_by_app");
        let ets = self.data.event_types.lock().unwrap();
        Ok(ets.iter().filter(|et| et.app_id() == app_id).count() as u64)
    }
}

pub struct FakeMessageStore {
    log: OperationLog,
    data: SharedMessageData,
}

impl FakeMessageStore {
    pub(crate) fn new(log: OperationLog, data: SharedMessageData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl MessageStore for FakeMessageStore {
    async fn insert_or_get_existing(
        &mut self,
        message: &Message,
        _org_id: &OrganizationId,
    ) -> Result<InsertMessageResult, ApplicationError> {
        self.log.record("message_store:insert_or_get_existing");
        let messages = self.data.messages.lock().unwrap();

        // Check for existing message with same app_id + idempotency_key that hasn't expired
        let existing = messages.iter().find(|m| {
            m.app_id() == message.app_id()
                && m.idempotency_key() == message.idempotency_key()
                && *m.idempotency_expires_at() > Utc::now()
        });

        if let Some(existing) = existing {
            return Ok(InsertMessageResult {
                message: existing.clone(),
                was_existing: true,
            });
        }

        drop(messages);
        self.data
            .messages
            .lock()
            .unwrap()
            .push(message.clone());

        Ok(InsertMessageResult {
            message: message.clone(),
            was_existing: false,
        })
    }

    async fn expire_idempotency_keys(
        &self,
        _now: DateTime<Utc>,
    ) -> Result<u64, ApplicationError> {
        self.log.record("message_store:expire_idempotency_keys");
        Ok(0)
    }
}

#[derive(Default, Clone)]
pub struct SharedAttemptData {
    pub attempts: Arc<std::sync::Mutex<Vec<Attempt>>>,
}

pub struct FakeAttemptStore {
    log: OperationLog,
    data: SharedAttemptData,
}

impl FakeAttemptStore {
    pub(crate) fn new(log: OperationLog, data: SharedAttemptData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl AttemptStore for FakeAttemptStore {
    async fn insert(&mut self, attempt: &Attempt) -> Result<(), ApplicationError> {
        self.log.record("attempt_store:insert");
        self.data.attempts.lock().unwrap().push(attempt.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &pigeon_domain::attempt::AttemptId,
        _org_id: &OrganizationId,
    ) -> Result<Option<Attempt>, ApplicationError> {
        self.log.record("attempt_store:find_by_id");
        let attempts = self.data.attempts.lock().unwrap();
        Ok(attempts.iter().find(|a| a.id() == id).cloned())
    }

    async fn save(&mut self, attempt: &Attempt) -> Result<(), ApplicationError> {
        self.log.record("attempt_store:save");
        let mut attempts = self.data.attempts.lock().unwrap();
        if let Some(existing) = attempts.iter_mut().find(|a| a.id() == attempt.id()) {
            *existing = attempt.clone();
        }
        Ok(())
    }
}

// --- Fake EndpointStore ---

pub struct FakeEndpointStore {
    log: OperationLog,
    data: SharedEndpointData,
}

impl FakeEndpointStore {
    pub(crate) fn new(log: OperationLog, data: SharedEndpointData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl EndpointStore for FakeEndpointStore {
    async fn insert(&mut self, endpoint: &Endpoint) -> Result<(), ApplicationError> {
        self.log.record("endpoint_store:insert");
        self.data
            .endpoints
            .lock()
            .unwrap()
            .push(endpoint.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &EndpointId,
        _org_id: &OrganizationId,
    ) -> Result<Option<Endpoint>, ApplicationError> {
        self.log.record("endpoint_store:find_by_id");
        let eps = self.data.endpoints.lock().unwrap();
        Ok(eps.iter().find(|ep| ep.id() == id).cloned())
    }

    async fn find_by_app_and_id(
        &self,
        id: &EndpointId,
        app_id: &pigeon_domain::application::ApplicationId,
    ) -> Result<Option<Endpoint>, ApplicationError> {
        self.log.record("endpoint_store:find_by_app_and_id");
        let eps = self.data.endpoints.lock().unwrap();
        Ok(eps
            .iter()
            .find(|ep| ep.id() == id && ep.app_id() == app_id)
            .cloned())
    }

    async fn save(&mut self, endpoint: &Endpoint) -> Result<(), ApplicationError> {
        self.log.record("endpoint_store:save");
        let mut eps = self.data.endpoints.lock().unwrap();
        if let Some(existing) = eps.iter_mut().find(|ep| ep.id() == endpoint.id()) {
            *existing = endpoint.clone();
        }
        Ok(())
    }

    async fn delete(&mut self, id: &EndpointId) -> Result<(), ApplicationError> {
        self.log.record("endpoint_store:delete");
        let mut eps = self.data.endpoints.lock().unwrap();
        eps.retain(|ep| ep.id() != id);
        Ok(())
    }
}

// --- Fake EndpointReadStore ---

pub struct FakeEndpointReadStore {
    log: OperationLog,
    endpoints: Vec<Endpoint>,
}

impl FakeEndpointReadStore {
    pub fn new(log: OperationLog, endpoints: Vec<Endpoint>) -> Self {
        Self { log, endpoints }
    }
}

#[async_trait]
impl EndpointReadStore for FakeEndpointReadStore {
    async fn find_enabled_by_app_and_event_type(
        &self,
        _app_id: &ApplicationId,
        _event_type_id: &EventTypeId,
        _org_id: &OrganizationId,
    ) -> Result<Vec<Endpoint>, ApplicationError> {
        self.log
            .record("endpoint_read_store:find_enabled_by_app_and_event_type");
        Ok(self.endpoints.clone())
    }

    async fn find_by_id(
        &self,
        id: &EndpointId,
        _org_id: &OrganizationId,
    ) -> Result<Option<Endpoint>, ApplicationError> {
        self.log.record("endpoint_read_store:find_by_id");
        Ok(self.endpoints.iter().find(|ep| ep.id() == id).cloned())
    }

    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        _org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Endpoint>, ApplicationError> {
        self.log.record("endpoint_read_store:list_by_app");
        let result = self
            .endpoints
            .iter()
            .filter(|ep| ep.app_id() == app_id)
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        _org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        self.log.record("endpoint_read_store:count_by_app");
        Ok(self
            .endpoints
            .iter()
            .filter(|ep| ep.app_id() == app_id)
            .count() as u64)
    }
}

// --- Fake OrganizationStore ---

pub struct FakeOrganizationStore {
    log: OperationLog,
    data: SharedOrganizationData,
}

impl FakeOrganizationStore {
    pub(crate) fn new(log: OperationLog, data: SharedOrganizationData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl OrganizationStore for FakeOrganizationStore {
    async fn insert(&mut self, organization: &Organization) -> Result<(), ApplicationError> {
        self.log.record("organization_store:insert");
        self.data
            .organizations
            .lock()
            .unwrap()
            .push(organization.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &OrganizationId,
    ) -> Result<Option<Organization>, ApplicationError> {
        self.log.record("organization_store:find_by_id");
        let orgs = self.data.organizations.lock().unwrap();
        Ok(orgs.iter().find(|o| o.id() == id).cloned())
    }

    async fn save(&mut self, organization: &Organization) -> Result<(), ApplicationError> {
        self.log.record("organization_store:save");
        let mut orgs = self.data.organizations.lock().unwrap();
        if let Some(existing) = orgs.iter_mut().find(|o| o.id() == organization.id()) {
            *existing = organization.clone();
        }
        Ok(())
    }

    async fn delete(&mut self, id: &OrganizationId) -> Result<(), ApplicationError> {
        self.log.record("organization_store:delete");
        let mut orgs = self.data.organizations.lock().unwrap();
        orgs.retain(|o| o.id() != id);
        Ok(())
    }
}

// --- Fake OrganizationReadStore ---

pub struct FakeOrganizationReadStore {
    log: OperationLog,
    data: SharedOrganizationData,
}

impl FakeOrganizationReadStore {
    pub fn new(log: OperationLog, data: SharedOrganizationData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl OrganizationReadStore for FakeOrganizationReadStore {
    async fn find_by_id(
        &self,
        id: &OrganizationId,
    ) -> Result<Option<Organization>, ApplicationError> {
        self.log.record("organization_read_store:find_by_id");
        let orgs = self.data.organizations.lock().unwrap();
        Ok(orgs.iter().find(|o| o.id() == id).cloned())
    }

    async fn find_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<Organization>, ApplicationError> {
        self.log.record("organization_read_store:find_by_slug");
        let orgs = self.data.organizations.lock().unwrap();
        Ok(orgs.iter().find(|o| o.slug() == slug).cloned())
    }

    async fn list(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Organization>, ApplicationError> {
        self.log.record("organization_read_store:list");
        let orgs = self.data.organizations.lock().unwrap();
        let result = orgs
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn count(&self) -> Result<u64, ApplicationError> {
        self.log.record("organization_read_store:count");
        let orgs = self.data.organizations.lock().unwrap();
        Ok(orgs.len() as u64)
    }
}

// --- Fake OidcConfigStore ---

pub struct FakeOidcConfigStore {
    log: OperationLog,
    data: SharedOidcConfigData,
}

impl FakeOidcConfigStore {
    pub(crate) fn new(log: OperationLog, data: SharedOidcConfigData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl OidcConfigStore for FakeOidcConfigStore {
    async fn insert(&mut self, config: &OidcConfig) -> Result<(), ApplicationError> {
        self.log.record("oidc_config_store:insert");
        self.data
            .oidc_configs
            .lock()
            .unwrap()
            .push(config.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &OidcConfigId,
    ) -> Result<Option<OidcConfig>, ApplicationError> {
        self.log.record("oidc_config_store:find_by_id");
        let configs = self.data.oidc_configs.lock().unwrap();
        Ok(configs.iter().find(|c| c.id() == id).cloned())
    }

    async fn count_by_org(&self, org_id: &OrganizationId) -> Result<u64, ApplicationError> {
        self.log.record("oidc_config_store:count_by_org");
        let configs = self.data.oidc_configs.lock().unwrap();
        Ok(configs.iter().filter(|c| c.org_id() == org_id).count() as u64)
    }

    async fn delete(&mut self, id: &OidcConfigId) -> Result<(), ApplicationError> {
        self.log.record("oidc_config_store:delete");
        let mut configs = self.data.oidc_configs.lock().unwrap();
        configs.retain(|c| c.id() != id);
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct SharedDeadLetterData {
    pub dead_letters: Arc<std::sync::Mutex<Vec<DeadLetter>>>,
}

pub struct FakeDeadLetterStore {
    log: OperationLog,
    data: SharedDeadLetterData,
}

impl FakeDeadLetterStore {
    pub(crate) fn new(log: OperationLog, data: SharedDeadLetterData) -> Self {
        Self { log, data }
    }
}

#[async_trait]
impl DeadLetterStore for FakeDeadLetterStore {
    async fn insert(&mut self, dead_letter: &DeadLetter) -> Result<(), ApplicationError> {
        self.log.record("dead_letter_store:insert");
        self.data.dead_letters.lock().unwrap().push(dead_letter.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &DeadLetterId,
        _org_id: &OrganizationId,
    ) -> Result<Option<DeadLetter>, ApplicationError> {
        self.log.record("dead_letter_store:find_by_id");
        let dls = self.data.dead_letters.lock().unwrap();
        Ok(dls.iter().find(|dl| dl.id() == id).cloned())
    }

    async fn save(&mut self, dead_letter: &DeadLetter) -> Result<(), ApplicationError> {
        self.log.record("dead_letter_store:save");
        let mut dls = self.data.dead_letters.lock().unwrap();
        if let Some(existing) = dls.iter_mut().find(|dl| dl.id() == dead_letter.id()) {
            *existing = dead_letter.clone();
        }
        Ok(())
    }
}

// --- Fake AuditStore ---

pub struct FakeAuditStore {
    log: OperationLog,
}

impl FakeAuditStore {
    pub fn new(log: OperationLog) -> Self {
        Self { log }
    }
}

#[async_trait]
impl AuditStore for FakeAuditStore {
    async fn record(&self, entry: AuditEntry) -> Result<(), ApplicationError> {
        self.log
            .record(&format!("audit:record:{}", entry.command_name));
        Ok(())
    }
}

// --- Fake UnitOfWork ---

pub struct FakeUnitOfWork {
    log: OperationLog,
    application_store: FakeApplicationStore,
    event_type_store: FakeEventTypeStore,
    endpoint_store: FakeEndpointStore,
    message_store: FakeMessageStore,
    attempt_store: FakeAttemptStore,
    dead_letter_store: FakeDeadLetterStore,
    organization_store: FakeOrganizationStore,
    oidc_config_store: FakeOidcConfigStore,
}

impl FakeUnitOfWork {
    pub(crate) fn new(
        log: OperationLog,
        app_data: SharedApplicationData,
        et_data: SharedEventTypeData,
        ep_data: SharedEndpointData,
        msg_data: SharedMessageData,
        att_data: SharedAttemptData,
        dl_data: SharedDeadLetterData,
        org_data: SharedOrganizationData,
        oidc_data: SharedOidcConfigData,
    ) -> Self {
        Self {
            application_store: FakeApplicationStore::new(log.clone(), app_data),
            event_type_store: FakeEventTypeStore::new(log.clone(), et_data),
            endpoint_store: FakeEndpointStore::new(log.clone(), ep_data),
            message_store: FakeMessageStore::new(log.clone(), msg_data),
            attempt_store: FakeAttemptStore::new(log.clone(), att_data),
            dead_letter_store: FakeDeadLetterStore::new(log.clone(), dl_data),
            organization_store: FakeOrganizationStore::new(log.clone(), org_data),
            oidc_config_store: FakeOidcConfigStore::new(log.clone(), oidc_data),
            log,
        }
    }
}

#[async_trait]
impl UnitOfWork for FakeUnitOfWork {
    async fn commit(self: Box<Self>) -> Result<(), ApplicationError> {
        self.log.record("uow:commit");
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError> {
        self.log.record("uow:rollback");
        Ok(())
    }

    fn application_store(&mut self) -> &mut dyn ApplicationStore {
        &mut self.application_store
    }

    fn event_type_store(&mut self) -> &mut dyn EventTypeStore {
        &mut self.event_type_store
    }

    fn message_store(&mut self) -> &mut dyn MessageStore {
        &mut self.message_store
    }

    fn attempt_store(&mut self) -> &mut dyn AttemptStore {
        &mut self.attempt_store
    }

    fn dead_letter_store(&mut self) -> &mut dyn DeadLetterStore {
        &mut self.dead_letter_store
    }

    fn endpoint_store(&mut self) -> &mut dyn EndpointStore {
        &mut self.endpoint_store
    }

    fn organization_store(&mut self) -> &mut dyn OrganizationStore {
        &mut self.organization_store
    }

    fn oidc_config_store(&mut self) -> &mut dyn OidcConfigStore {
        &mut self.oidc_config_store
    }

    fn emit_event(&mut self, event: pigeon_domain::event::DomainEvent) {
        self.log.record(&format!("uow:emit_event:{}", event.event_type()));
    }
}

// --- Fake UnitOfWorkFactory ---

pub struct FakeUnitOfWorkFactory {
    log: OperationLog,
    app_data: SharedApplicationData,
    et_data: SharedEventTypeData,
    ep_data: SharedEndpointData,
    msg_data: SharedMessageData,
    att_data: SharedAttemptData,
    dl_data: SharedDeadLetterData,
    org_data: SharedOrganizationData,
    oidc_data: SharedOidcConfigData,
}

impl FakeUnitOfWorkFactory {
    pub fn new(log: OperationLog) -> Self {
        Self {
            log,
            app_data: SharedApplicationData::default(),
            et_data: SharedEventTypeData::default(),
            ep_data: SharedEndpointData::default(),
            msg_data: SharedMessageData::default(),
            att_data: SharedAttemptData::default(),
            dl_data: SharedDeadLetterData::default(),
            org_data: SharedOrganizationData::default(),
            oidc_data: SharedOidcConfigData::default(),
        }
    }

    pub fn with_data(log: OperationLog, app_data: SharedApplicationData) -> Self {
        Self {
            log,
            app_data,
            et_data: SharedEventTypeData::default(),
            ep_data: SharedEndpointData::default(),
            msg_data: SharedMessageData::default(),
            att_data: SharedAttemptData::default(),
            dl_data: SharedDeadLetterData::default(),
            org_data: SharedOrganizationData::default(),
            oidc_data: SharedOidcConfigData::default(),
        }
    }

    pub fn new_with_messages(log: OperationLog, msg_data: SharedMessageData) -> Self {
        Self {
            log,
            app_data: SharedApplicationData::default(),
            et_data: SharedEventTypeData::default(),
            ep_data: SharedEndpointData::default(),
            msg_data,
            att_data: SharedAttemptData::default(),
            dl_data: SharedDeadLetterData::default(),
            org_data: SharedOrganizationData::default(),
            oidc_data: SharedOidcConfigData::default(),
        }
    }

    pub fn with_event_type_data(
        log: OperationLog,
        et_data: SharedEventTypeData,
    ) -> Self {
        Self {
            log,
            app_data: SharedApplicationData::default(),
            et_data,
            ep_data: SharedEndpointData::default(),
            msg_data: SharedMessageData::default(),
            att_data: SharedAttemptData::default(),
            dl_data: SharedDeadLetterData::default(),
            org_data: SharedOrganizationData::default(),
            oidc_data: SharedOidcConfigData::default(),
        }
    }

    pub fn with_endpoint_data(
        log: OperationLog,
        ep_data: SharedEndpointData,
    ) -> Self {
        Self {
            log,
            app_data: SharedApplicationData::default(),
            et_data: SharedEventTypeData::default(),
            ep_data,
            msg_data: SharedMessageData::default(),
            att_data: SharedAttemptData::default(),
            dl_data: SharedDeadLetterData::default(),
            org_data: SharedOrganizationData::default(),
            oidc_data: SharedOidcConfigData::default(),
        }
    }

    pub fn with_organization_data(
        log: OperationLog,
        org_data: SharedOrganizationData,
    ) -> Self {
        Self {
            log,
            app_data: SharedApplicationData::default(),
            et_data: SharedEventTypeData::default(),
            ep_data: SharedEndpointData::default(),
            msg_data: SharedMessageData::default(),
            att_data: SharedAttemptData::default(),
            dl_data: SharedDeadLetterData::default(),
            org_data,
            oidc_data: SharedOidcConfigData::default(),
        }
    }

    pub fn with_oidc_config_data(
        log: OperationLog,
        oidc_data: SharedOidcConfigData,
    ) -> Self {
        Self {
            log,
            app_data: SharedApplicationData::default(),
            et_data: SharedEventTypeData::default(),
            ep_data: SharedEndpointData::default(),
            msg_data: SharedMessageData::default(),
            att_data: SharedAttemptData::default(),
            dl_data: SharedDeadLetterData::default(),
            org_data: SharedOrganizationData::default(),
            oidc_data,
        }
    }

    pub fn oidc_config_data(&self) -> &SharedOidcConfigData {
        &self.oidc_data
    }

    pub fn app_data(&self) -> &SharedApplicationData {
        &self.app_data
    }

    pub fn event_type_data(&self) -> &SharedEventTypeData {
        &self.et_data
    }

    pub fn endpoint_data(&self) -> &SharedEndpointData {
        &self.ep_data
    }

    pub fn with_dead_letter_data(
        log: OperationLog,
        dl_data: SharedDeadLetterData,
    ) -> Self {
        Self {
            log,
            app_data: SharedApplicationData::default(),
            et_data: SharedEventTypeData::default(),
            ep_data: SharedEndpointData::default(),
            msg_data: SharedMessageData::default(),
            att_data: SharedAttemptData::default(),
            dl_data,
            org_data: SharedOrganizationData::default(),
            oidc_data: SharedOidcConfigData::default(),
        }
    }

    pub fn dead_letter_data(&self) -> &SharedDeadLetterData {
        &self.dl_data
    }

    pub fn organization_data(&self) -> &SharedOrganizationData {
        &self.org_data
    }
}

#[async_trait]
impl UnitOfWorkFactory for FakeUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn UnitOfWork>, ApplicationError> {
        self.log.record("uow_factory:begin");
        Ok(Box::new(FakeUnitOfWork::new(
            self.log.clone(),
            self.app_data.clone(),
            self.et_data.clone(),
            self.ep_data.clone(),
            self.msg_data.clone(),
            self.att_data.clone(),
            self.dl_data.clone(),
            self.org_data.clone(),
            self.oidc_data.clone(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn factory_creates_unit_of_work() {
        let log = OperationLog::new();
        let factory = FakeUnitOfWorkFactory::new(log.clone());

        let _uow = factory.begin().await.unwrap();

        assert_eq!(log.entries(), vec!["uow_factory:begin"]);
    }

    #[tokio::test]
    async fn commit_is_recorded() {
        let log = OperationLog::new();
        let factory = FakeUnitOfWorkFactory::new(log.clone());
        let uow = factory.begin().await.unwrap();

        uow.commit().await.unwrap();

        assert_eq!(log.entries(), vec!["uow_factory:begin", "uow:commit"]);
    }

    #[tokio::test]
    async fn rollback_is_recorded() {
        let log = OperationLog::new();
        let factory = FakeUnitOfWorkFactory::new(log.clone());
        let uow = factory.begin().await.unwrap();

        uow.rollback().await.unwrap();

        assert_eq!(log.entries(), vec!["uow_factory:begin", "uow:rollback"]);
    }

    #[tokio::test]
    async fn store_accessors_return_usable_stores() {
        let log = OperationLog::new();
        let factory = FakeUnitOfWorkFactory::new(log.clone());
        let mut uow = factory.begin().await.unwrap();

        // Access each store — verifies the trait returns concrete implementations
        let _msg_store = uow.message_store();
        let _att_store = uow.attempt_store();
        let _dl_store = uow.dead_letter_store();

        // Stores are accessible without panicking; UoW is still usable
        uow.commit().await.unwrap();

        assert_eq!(log.entries(), vec!["uow_factory:begin", "uow:commit"]);
    }

    #[tokio::test]
    async fn stores_share_operation_log_with_uow() {
        let log = OperationLog::new();
        let factory = FakeUnitOfWorkFactory::new(log.clone());
        let mut uow = factory.begin().await.unwrap();

        // Use a store, then commit — both appear in the same log
        uow.dead_letter_store()
            .insert(&DeadLetter::new(
                pigeon_domain::message::MessageId::new(),
                pigeon_domain::endpoint::EndpointId::new(),
                ApplicationId::new(),
                None,
                None,
            ))
            .await
            .unwrap();

        uow.commit().await.unwrap();

        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "dead_letter_store:insert",
                "uow:commit"
            ]
        );
    }

    #[tokio::test]
    async fn application_data_persists_across_uow_instances() {
        let log = OperationLog::new();
        let factory = FakeUnitOfWorkFactory::new(log.clone());

        let app = Application::new(OrganizationId::new(), "test-app".into(), "app_123".into()).unwrap();
        let id = app.id().clone();

        // Insert in first UoW
        {
            let mut uow = factory.begin().await.unwrap();
            uow.application_store().insert(&app).await.unwrap();
            uow.commit().await.unwrap();
        }

        // Find in second UoW
        {
            let mut uow = factory.begin().await.unwrap();
            let found = uow.application_store().find_by_id(&id).await.unwrap();
            assert!(found.is_some());
            assert_eq!(found.unwrap().name(), "test-app");
        }
    }
}
