use uuid::Uuid;

use crate::application::ApplicationId;
use crate::endpoint::EndpointId;
use crate::message::MessageId;

#[derive(Debug, Clone)]
pub enum DomainEvent {
    DeadLettered {
        message_id: MessageId,
        endpoint_id: EndpointId,
        app_id: ApplicationId,
    },
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::DeadLettered { .. } => "dead_lettered",
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            DomainEvent::DeadLettered {
                message_id,
                endpoint_id,
                app_id,
            } => serde_json::json!({
                "message_id": message_id.as_uuid(),
                "endpoint_id": endpoint_id.as_uuid(),
                "app_id": app_id.as_uuid(),
            }),
        }
    }

    pub fn from_outbox(
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Option<Self> {
        match event_type {
            "dead_lettered" => {
                let message_id = payload.get("message_id")?.as_str()?;
                let endpoint_id = payload.get("endpoint_id")?.as_str()?;
                let app_id = payload.get("app_id")?.as_str()?;
                Some(DomainEvent::DeadLettered {
                    message_id: MessageId::from_uuid(Uuid::parse_str(message_id).ok()?),
                    endpoint_id: EndpointId::from_uuid(Uuid::parse_str(endpoint_id).ok()?),
                    app_id: ApplicationId::from_uuid(Uuid::parse_str(app_id).ok()?),
                })
            }
            _ => None,
        }
    }
}
