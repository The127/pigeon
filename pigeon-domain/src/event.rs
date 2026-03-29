use uuid::Uuid;

use crate::application::ApplicationId;
use crate::attempt::AttemptId;
use crate::dead_letter::DeadLetterId;
use crate::endpoint::EndpointId;
use crate::event_type::EventTypeId;
use crate::message::MessageId;

#[derive(Debug, Clone, PartialEq)]
pub enum DomainEvent {
    MessageCreated {
        message_id: MessageId,
        app_id: ApplicationId,
        event_type_id: EventTypeId,
        attempts_created: u32,
    },
    MessageRetriggered {
        message_id: MessageId,
        attempts_created: u32,
    },
    DeadLettered {
        message_id: MessageId,
        endpoint_id: EndpointId,
        app_id: ApplicationId,
    },
    DeadLetterReplayed {
        dead_letter_id: DeadLetterId,
        message_id: MessageId,
        endpoint_id: EndpointId,
    },
    AttemptSucceeded {
        attempt_id: AttemptId,
        message_id: MessageId,
        endpoint_id: EndpointId,
        response_code: u16,
        duration_ms: i64,
    },
    AttemptFailed {
        attempt_id: AttemptId,
        message_id: MessageId,
        endpoint_id: EndpointId,
        response_code: Option<u16>,
        duration_ms: i64,
        will_retry: bool,
    },
    EndpointUpdated {
        endpoint_id: EndpointId,
        app_id: ApplicationId,
        enabled: bool,
    },
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::MessageCreated { .. } => "message_created",
            DomainEvent::MessageRetriggered { .. } => "message_retriggered",
            DomainEvent::DeadLettered { .. } => "dead_lettered",
            DomainEvent::AttemptSucceeded { .. } => "attempt_succeeded",
            DomainEvent::AttemptFailed { .. } => "attempt_failed",
            DomainEvent::DeadLetterReplayed { .. } => "dead_letter_replayed",
            DomainEvent::EndpointUpdated { .. } => "endpoint_updated",
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            DomainEvent::MessageCreated {
                message_id,
                app_id,
                event_type_id,
                attempts_created,
            } => serde_json::json!({
                "message_id": message_id.as_uuid(),
                "app_id": app_id.as_uuid(),
                "event_type_id": event_type_id.as_uuid(),
                "attempts_created": attempts_created,
            }),
            DomainEvent::MessageRetriggered {
                message_id,
                attempts_created,
            } => serde_json::json!({
                "message_id": message_id.as_uuid(),
                "attempts_created": attempts_created,
            }),
            DomainEvent::DeadLettered {
                message_id,
                endpoint_id,
                app_id,
            } => serde_json::json!({
                "message_id": message_id.as_uuid(),
                "endpoint_id": endpoint_id.as_uuid(),
                "app_id": app_id.as_uuid(),
            }),
            DomainEvent::AttemptSucceeded {
                attempt_id,
                message_id,
                endpoint_id,
                response_code,
                duration_ms,
            } => serde_json::json!({
                "attempt_id": attempt_id.as_uuid(),
                "message_id": message_id.as_uuid(),
                "endpoint_id": endpoint_id.as_uuid(),
                "response_code": response_code,
                "duration_ms": duration_ms,
            }),
            DomainEvent::AttemptFailed {
                attempt_id,
                message_id,
                endpoint_id,
                response_code,
                duration_ms,
                will_retry,
            } => serde_json::json!({
                "attempt_id": attempt_id.as_uuid(),
                "message_id": message_id.as_uuid(),
                "endpoint_id": endpoint_id.as_uuid(),
                "response_code": response_code,
                "duration_ms": duration_ms,
                "will_retry": will_retry,
            }),
            DomainEvent::DeadLetterReplayed {
                dead_letter_id,
                message_id,
                endpoint_id,
            } => serde_json::json!({
                "dead_letter_id": dead_letter_id.as_uuid(),
                "message_id": message_id.as_uuid(),
                "endpoint_id": endpoint_id.as_uuid(),
            }),
            DomainEvent::EndpointUpdated {
                endpoint_id,
                app_id,
                enabled,
            } => serde_json::json!({
                "endpoint_id": endpoint_id.as_uuid(),
                "app_id": app_id.as_uuid(),
                "enabled": enabled,
            }),
        }
    }

    pub fn from_outbox(event_type: &str, payload: &serde_json::Value) -> Option<Self> {
        match event_type {
            "message_created" => {
                let message_id = parse_uuid(payload, "message_id")?;
                let app_id = parse_uuid(payload, "app_id")?;
                let event_type_id = parse_uuid(payload, "event_type_id")?;
                let attempts_created = payload.get("attempts_created")?.as_u64()? as u32;
                Some(DomainEvent::MessageCreated {
                    message_id: MessageId::from_uuid(message_id),
                    app_id: ApplicationId::from_uuid(app_id),
                    event_type_id: EventTypeId::from_uuid(event_type_id),
                    attempts_created,
                })
            }
            "message_retriggered" => {
                let message_id = parse_uuid(payload, "message_id")?;
                let attempts_created = payload.get("attempts_created")?.as_u64()? as u32;
                Some(DomainEvent::MessageRetriggered {
                    message_id: MessageId::from_uuid(message_id),
                    attempts_created,
                })
            }
            "dead_lettered" => {
                let message_id = parse_uuid(payload, "message_id")?;
                let endpoint_id = parse_uuid(payload, "endpoint_id")?;
                let app_id = parse_uuid(payload, "app_id")?;
                Some(DomainEvent::DeadLettered {
                    message_id: MessageId::from_uuid(message_id),
                    endpoint_id: EndpointId::from_uuid(endpoint_id),
                    app_id: ApplicationId::from_uuid(app_id),
                })
            }
            "attempt_succeeded" => {
                let attempt_id = parse_uuid(payload, "attempt_id")?;
                let message_id = parse_uuid(payload, "message_id")?;
                let endpoint_id = parse_uuid(payload, "endpoint_id")?;
                let response_code = payload.get("response_code")?.as_u64()? as u16;
                let duration_ms = payload.get("duration_ms")?.as_i64()?;
                Some(DomainEvent::AttemptSucceeded {
                    attempt_id: AttemptId::from_uuid(attempt_id),
                    message_id: MessageId::from_uuid(message_id),
                    endpoint_id: EndpointId::from_uuid(endpoint_id),
                    response_code,
                    duration_ms,
                })
            }
            "attempt_failed" => {
                let attempt_id = parse_uuid(payload, "attempt_id")?;
                let message_id = parse_uuid(payload, "message_id")?;
                let endpoint_id = parse_uuid(payload, "endpoint_id")?;
                let response_code = payload
                    .get("response_code")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16);
                let duration_ms = payload.get("duration_ms")?.as_i64()?;
                let will_retry = payload.get("will_retry")?.as_bool()?;
                Some(DomainEvent::AttemptFailed {
                    attempt_id: AttemptId::from_uuid(attempt_id),
                    message_id: MessageId::from_uuid(message_id),
                    endpoint_id: EndpointId::from_uuid(endpoint_id),
                    response_code,
                    duration_ms,
                    will_retry,
                })
            }
            "dead_letter_replayed" => {
                let dead_letter_id = parse_uuid(payload, "dead_letter_id")?;
                let message_id = parse_uuid(payload, "message_id")?;
                let endpoint_id = parse_uuid(payload, "endpoint_id")?;
                Some(DomainEvent::DeadLetterReplayed {
                    dead_letter_id: DeadLetterId::from_uuid(dead_letter_id),
                    message_id: MessageId::from_uuid(message_id),
                    endpoint_id: EndpointId::from_uuid(endpoint_id),
                })
            }
            "endpoint_updated" => {
                let endpoint_id = parse_uuid(payload, "endpoint_id")?;
                let app_id = parse_uuid(payload, "app_id")?;
                let enabled = payload.get("enabled")?.as_bool()?;
                Some(DomainEvent::EndpointUpdated {
                    endpoint_id: EndpointId::from_uuid(endpoint_id),
                    app_id: ApplicationId::from_uuid(app_id),
                    enabled,
                })
            }
            _ => None,
        }
    }
}

fn parse_uuid(payload: &serde_json::Value, field: &str) -> Option<Uuid> {
    Uuid::parse_str(payload.get(field)?.as_str()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_message_created() {
        let event = DomainEvent::MessageCreated {
            message_id: MessageId::new(),
            app_id: ApplicationId::new(),
            event_type_id: EventTypeId::new(),
            attempts_created: 3,
        };
        let json = event.to_json();
        let restored = DomainEvent::from_outbox(event.event_type(), &json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn roundtrip_message_retriggered() {
        let event = DomainEvent::MessageRetriggered {
            message_id: MessageId::new(),
            attempts_created: 2,
        };
        let json = event.to_json();
        let restored = DomainEvent::from_outbox(event.event_type(), &json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn roundtrip_dead_lettered() {
        let event = DomainEvent::DeadLettered {
            message_id: MessageId::new(),
            endpoint_id: EndpointId::new(),
            app_id: ApplicationId::new(),
        };
        let json = event.to_json();
        let restored = DomainEvent::from_outbox(event.event_type(), &json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn roundtrip_dead_letter_replayed() {
        let event = DomainEvent::DeadLetterReplayed {
            dead_letter_id: DeadLetterId::new(),
            message_id: MessageId::new(),
            endpoint_id: EndpointId::new(),
        };
        let json = event.to_json();
        let restored = DomainEvent::from_outbox(event.event_type(), &json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn roundtrip_attempt_succeeded() {
        let event = DomainEvent::AttemptSucceeded {
            attempt_id: AttemptId::new(),
            message_id: MessageId::new(),
            endpoint_id: EndpointId::new(),
            response_code: 200,
            duration_ms: 150,
        };
        let json = event.to_json();
        let restored = DomainEvent::from_outbox(event.event_type(), &json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn roundtrip_attempt_failed_with_response() {
        let event = DomainEvent::AttemptFailed {
            attempt_id: AttemptId::new(),
            message_id: MessageId::new(),
            endpoint_id: EndpointId::new(),
            response_code: Some(500),
            duration_ms: 42,
            will_retry: true,
        };
        let json = event.to_json();
        let restored = DomainEvent::from_outbox(event.event_type(), &json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn roundtrip_attempt_failed_network_error() {
        let event = DomainEvent::AttemptFailed {
            attempt_id: AttemptId::new(),
            message_id: MessageId::new(),
            endpoint_id: EndpointId::new(),
            response_code: None,
            duration_ms: 5,
            will_retry: false,
        };
        let json = event.to_json();
        let restored = DomainEvent::from_outbox(event.event_type(), &json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn roundtrip_endpoint_updated() {
        let event = DomainEvent::EndpointUpdated {
            endpoint_id: EndpointId::new(),
            app_id: ApplicationId::new(),
            enabled: false,
        };
        let json = event.to_json();
        let restored = DomainEvent::from_outbox(event.event_type(), &json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn unknown_event_type_returns_none() {
        assert!(DomainEvent::from_outbox("unknown", &serde_json::json!({})).is_none());
    }
}
