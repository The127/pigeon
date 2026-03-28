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
