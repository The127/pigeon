use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEntryId(Uuid);

impl OutboxEntryId {
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

impl Default for OutboxEntryId {
    fn default() -> Self {
        Self::new()
    }
}
