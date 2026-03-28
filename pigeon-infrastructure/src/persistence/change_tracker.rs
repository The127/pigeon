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
            match change {
                Change::InsertMessage(msg) => {
                    events.push(DomainEvent::MessageCreated {
                        message_id: msg.id().clone(),
                        app_id: msg.app_id().clone(),
                        event_type_id: msg.event_type_id().clone(),
                    });
                }
                Change::InsertDeadLetter(dl) => {
                    events.push(DomainEvent::DeadLettered {
                        message_id: dl.message_id().clone(),
                        endpoint_id: dl.endpoint_id().clone(),
                        app_id: dl.app_id().clone(),
                    });
                }
                Change::SaveDeadLetter(dl) if dl.replayed_at().is_some() => {
                    events.push(DomainEvent::DeadLetterReplayed {
                        dead_letter_id: dl.id().clone(),
                        message_id: dl.message_id().clone(),
                        endpoint_id: dl.endpoint_id().clone(),
                    });
                }
                Change::SaveEndpoint(ep) => {
                    events.push(DomainEvent::EndpointUpdated {
                        endpoint_id: ep.id().clone(),
                        app_id: ep.app_id().clone(),
                        enabled: ep.enabled(),
                    });
                }
                _ => {}
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pigeon_domain::dead_letter::DeadLetter;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::message::Message;

    #[test]
    fn insert_message_emits_message_created() {
        let mut tracker = ChangeTracker::new();
        let msg = Message::new(
            ApplicationId::new(),
            EventTypeId::new(),
            serde_json::json!({"test": true}),
            None,
            chrono::Duration::hours(1),
        )
        .unwrap();

        tracker.record(Change::InsertMessage(msg.clone()));
        let events = tracker.collect_events();

        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::MessageCreated {
                message_id,
                app_id,
                event_type_id,
            } => {
                assert_eq!(message_id, msg.id());
                assert_eq!(app_id, msg.app_id());
                assert_eq!(event_type_id, msg.event_type_id());
            }
            _ => panic!("expected MessageCreated"),
        }
    }

    #[test]
    fn insert_dead_letter_emits_dead_lettered() {
        let mut tracker = ChangeTracker::new();
        let dl = DeadLetter::new(
            pigeon_domain::message::MessageId::new(),
            EndpointId::new(),
            ApplicationId::new(),
            Some(500),
            Some("error".into()),
        );

        tracker.record(Change::InsertDeadLetter(dl.clone()));
        let events = tracker.collect_events();

        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::DeadLettered {
                message_id,
                endpoint_id,
                app_id,
            } => {
                assert_eq!(message_id, dl.message_id());
                assert_eq!(endpoint_id, dl.endpoint_id());
                assert_eq!(app_id, dl.app_id());
            }
            _ => panic!("expected DeadLettered"),
        }
    }

    #[test]
    fn save_dead_letter_with_replayed_at_emits_replayed() {
        let mut tracker = ChangeTracker::new();
        let mut dl = DeadLetter::new(
            pigeon_domain::message::MessageId::new(),
            EndpointId::new(),
            ApplicationId::new(),
            None,
            None,
        );
        dl.mark_replayed();

        tracker.record(Change::SaveDeadLetter(dl.clone()));
        let events = tracker.collect_events();

        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::DeadLetterReplayed {
                dead_letter_id,
                message_id,
                endpoint_id,
            } => {
                assert_eq!(dead_letter_id, dl.id());
                assert_eq!(message_id, dl.message_id());
                assert_eq!(endpoint_id, dl.endpoint_id());
            }
            _ => panic!("expected DeadLetterReplayed"),
        }
    }

    #[test]
    fn save_dead_letter_without_replayed_at_emits_nothing() {
        let mut tracker = ChangeTracker::new();
        let dl = DeadLetter::new(
            pigeon_domain::message::MessageId::new(),
            EndpointId::new(),
            ApplicationId::new(),
            None,
            None,
        );

        tracker.record(Change::SaveDeadLetter(dl));
        let events = tracker.collect_events();

        assert!(events.is_empty());
    }

    #[test]
    fn save_endpoint_emits_endpoint_updated() {
        let mut tracker = ChangeTracker::new();
        let ep = Endpoint::new(
            ApplicationId::new(),
            "https://example.com/hook".into(),
            "whsec_test".into(),
            vec![EventTypeId::new()],
        )
        .unwrap();

        tracker.record(Change::SaveEndpoint(ep.clone()));
        let events = tracker.collect_events();

        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::EndpointUpdated {
                endpoint_id,
                app_id,
                enabled,
            } => {
                assert_eq!(endpoint_id, ep.id());
                assert_eq!(app_id, ep.app_id());
                assert!(enabled);
            }
            _ => panic!("expected EndpointUpdated"),
        }
    }

    #[test]
    fn no_events_for_non_emitting_changes() {
        let mut tracker = ChangeTracker::new();
        let app = Application::new(
            OrganizationId::new(),
            "test".into(),
            "uid".into(),
        )
        .unwrap();

        tracker.record(Change::InsertApplication(app));
        let events = tracker.collect_events();

        assert!(events.is_empty());
    }

    #[test]
    fn multiple_changes_emit_multiple_events() {
        let mut tracker = ChangeTracker::new();

        let msg = Message::new(
            ApplicationId::new(),
            EventTypeId::new(),
            serde_json::json!({"x": 1}),
            None,
            chrono::Duration::hours(1),
        )
        .unwrap();

        let dl = DeadLetter::new(
            pigeon_domain::message::MessageId::new(),
            EndpointId::new(),
            ApplicationId::new(),
            None,
            None,
        );

        tracker.record(Change::InsertMessage(msg));
        tracker.record(Change::InsertDeadLetter(dl));

        let events = tracker.collect_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type(), "message_created");
        assert_eq!(events[1].event_type(), "dead_lettered");
    }
}
