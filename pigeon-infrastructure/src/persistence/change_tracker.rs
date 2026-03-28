use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::attempt::Attempt;
use pigeon_domain::dead_letter::DeadLetter;
use pigeon_domain::endpoint::{Endpoint, EndpointId};
use pigeon_domain::event::DomainEvent;
use pigeon_domain::event_type::{EventType, EventTypeId};
use pigeon_domain::message::Message;
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId};
use pigeon_domain::organization::{Organization, OrganizationId};

pub(crate) enum Change {
    InsertApplication(Application),
    SaveApplication(Application),
    DeleteApplication(ApplicationId),
    InsertEventType(EventType),
    SaveEventType(EventType),
    DeleteEventType(EventTypeId),
    InsertEndpoint(Endpoint),
    SaveEndpoint(Endpoint),
    DeleteEndpoint(EndpointId),
    InsertMessage(Message),
    InsertAttempt(Attempt),
    SaveAttempt(Attempt),
    InsertDeadLetter(DeadLetter),
    SaveDeadLetter(DeadLetter),
    InsertOrganization(Organization),
    SaveOrganization(Organization),
    DeleteOrganization(OrganizationId),
    InsertOidcConfig(OidcConfig),
    DeleteOidcConfig(OidcConfigId),
}

pub(crate) struct ChangeTracker {
    changes: Vec<Change>,
}

impl ChangeTracker {
    pub(crate) fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    pub(crate) fn record(&mut self, change: Change) {
        self.changes.push(change);
    }

    pub(crate) fn drain(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }

    /// Find the most recent pending state for an application by id.
    ///
    /// Returns:
    /// - `Some(Some(app))` — pending insert or save
    /// - `Some(None)` — pending delete
    /// - `None` — no pending change, caller should query DB
    pub(crate) fn find_pending_application(
        &self,
        id: &ApplicationId,
    ) -> Option<Option<&Application>> {
        for change in self.changes.iter().rev() {
            match change {
                Change::InsertApplication(app) | Change::SaveApplication(app)
                    if app.id() == id =>
                {
                    return Some(Some(app));
                }
                Change::DeleteApplication(del_id) if del_id == id => {
                    return Some(None);
                }
                _ => continue,
            }
        }
        None
    }

    /// Find the most recent pending state for an event type by id.
    ///
    /// Returns:
    /// - `Some(Some(et))` — pending insert or save
    /// - `Some(None)` — pending delete
    /// - `None` — no pending change, caller should query DB
    pub(crate) fn find_pending_event_type(
        &self,
        id: &EventTypeId,
    ) -> Option<Option<&EventType>> {
        for change in self.changes.iter().rev() {
            match change {
                Change::InsertEventType(et) | Change::SaveEventType(et)
                    if et.id() == id =>
                {
                    return Some(Some(et));
                }
                Change::DeleteEventType(del_id) if del_id == id => {
                    return Some(None);
                }
                _ => continue,
            }
        }
        None
    }

    /// Find the most recent pending state for an organization by id.
    ///
    /// Returns:
    /// - `Some(Some(org))` — pending insert or save
    /// - `Some(None)` — pending delete
    /// - `None` — no pending change, caller should query DB
    pub(crate) fn find_pending_organization(
        &self,
        id: &OrganizationId,
    ) -> Option<Option<&Organization>> {
        for change in self.changes.iter().rev() {
            match change {
                Change::InsertOrganization(org) | Change::SaveOrganization(org)
                    if org.id() == id =>
                {
                    return Some(Some(org));
                }
                Change::DeleteOrganization(del_id) if del_id == id => {
                    return Some(None);
                }
                _ => continue,
            }
        }
        None
    }

    /// Find the most recent pending state for an OIDC config by id.
    ///
    /// Returns:
    /// - `Some(Some(config))` — pending insert
    /// - `Some(None)` — pending delete
    /// - `None` — no pending change, caller should query DB
    pub(crate) fn find_pending_oidc_config(
        &self,
        id: &OidcConfigId,
    ) -> Option<Option<&OidcConfig>> {
        for change in self.changes.iter().rev() {
            match change {
                Change::InsertOidcConfig(config) if config.id() == id => {
                    return Some(Some(config));
                }
                Change::DeleteOidcConfig(del_id) if del_id == id => {
                    return Some(None);
                }
                _ => continue,
            }
        }
        None
    }

    /// Find the most recent pending state for an endpoint by id.
    ///
    /// Returns:
    /// - `Some(Some(ep))` — pending insert or save
    /// - `Some(None)` — pending delete
    /// - `None` — no pending change, caller should query DB
    pub(crate) fn find_pending_endpoint(
        &self,
        id: &EndpointId,
    ) -> Option<Option<&Endpoint>> {
        for change in self.changes.iter().rev() {
            match change {
                Change::InsertEndpoint(ep) | Change::SaveEndpoint(ep)
                    if ep.id() == id =>
                {
                    return Some(Some(ep));
                }
                Change::DeleteEndpoint(del_id) if del_id == id => {
                    return Some(None);
                }
                _ => continue,
            }
        }
        None
    }

    /// Extract domain events from pending changes.
    /// Called during UoW commit to populate the transactional outbox.
    pub(crate) fn collect_events(&self) -> Vec<DomainEvent> {
        let mut events = Vec::new();
        for change in &self.changes {
            if let Change::InsertDeadLetter(dl) = change {
                events.push(DomainEvent::DeadLettered {
                    message_id: dl.message_id().clone(),
                    endpoint_id: dl.endpoint_id().clone(),
                    app_id: dl.app_id().clone(),
                });
            }
        }
        events
    }
}
