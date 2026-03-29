use pigeon_domain::message::Message;

/// A message enriched with delivery status from the projection table.
#[derive(Debug, Clone)]
pub struct MessageWithStatus {
    pub message: Message,
    pub attempts_created: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub dead_lettered: u32,
}
