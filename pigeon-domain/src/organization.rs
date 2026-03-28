use chrono::{DateTime, Utc};
use pigeon_macros::Reconstitute;
use uuid::Uuid;

use crate::version::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationId(Uuid);

impl Default for OrganizationId {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganizationId {
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

#[derive(Debug, Clone, Reconstitute)]
pub struct Organization {
    id: OrganizationId,
    name: String,
    slug: String,
    created_at: DateTime<Utc>,
    version: Version,
}

#[derive(Debug, thiserror::Error)]
pub enum OrganizationError {
    #[error("organization name must not be empty")]
    EmptyName,
    #[error("organization slug must not be empty")]
    EmptySlug,
    #[error("organization slug contains invalid characters")]
    InvalidSlug,
}

impl Organization {
    pub fn new(name: String, slug: String) -> Result<Self, OrganizationError> {
        if name.trim().is_empty() {
            return Err(OrganizationError::EmptyName);
        }
        if slug.trim().is_empty() {
            return Err(OrganizationError::EmptySlug);
        }
        if !slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(OrganizationError::InvalidSlug);
        }

        Ok(Self {
            id: OrganizationId::new(),
            name,
            slug,
            created_at: Utc::now(),
            version: Version::new(0),
        })
    }

    pub fn id(&self) -> &OrganizationId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn rename(&mut self, name: String) -> Result<(), OrganizationError> {
        if name.trim().is_empty() {
            return Err(OrganizationError::EmptyName);
        }
        self.name = name;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::any_organization;

    #[test]
    fn create_organization_with_valid_fields() {
        let org = Organization::new("my-org".into(), "my-org".into()).unwrap();

        assert_eq!(org.name(), "my-org");
        assert_eq!(org.slug(), "my-org");
    }

    #[test]
    fn reject_empty_name() {
        let result = Organization::new("".into(), "my-org".into());
        assert!(result.is_err());
    }

    #[test]
    fn reject_whitespace_only_name() {
        let result = Organization::new("   ".into(), "my-org".into());
        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_slug() {
        let result = Organization::new("my-org".into(), "".into());
        assert!(result.is_err());
    }

    #[test]
    fn reject_whitespace_only_slug() {
        let result = Organization::new("my-org".into(), "   ".into());
        assert!(result.is_err());
    }

    #[test]
    fn reject_slug_with_uppercase() {
        let result = Organization::new("my-org".into(), "My-Org".into());
        assert!(result.is_err());
    }

    #[test]
    fn reject_slug_with_spaces() {
        let result = Organization::new("my-org".into(), "my org".into());
        assert!(result.is_err());
    }

    #[test]
    fn accept_slug_with_hyphens_and_digits() {
        let org = Organization::new("my-org".into(), "my-org-123".into()).unwrap();
        assert_eq!(org.slug(), "my-org-123");
    }

    #[test]
    fn new_organization_has_version_zero() {
        let org = any_organization();
        assert_eq!(org.version(), Version::new(0));
    }

    #[test]
    fn organization_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = OrganizationId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn organization_id_new_is_unique() {
        let id1 = OrganizationId::new();
        let id2 = OrganizationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn rename_updates_name() {
        let mut org = any_organization();
        org.rename("new-name".into()).unwrap();
        assert_eq!(org.name(), "new-name");
    }

    #[test]
    fn rename_rejects_empty_name() {
        let mut org = any_organization();
        let result = org.rename("".into());
        assert!(result.is_err());
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let state = OrganizationState::fake();

        let org = Organization::reconstitute(OrganizationState {
            id: state.id.clone(),
            name: state.name.clone(),
            slug: state.slug.clone(),
            created_at: state.created_at,
            version: state.version,
        });

        assert_eq!(*org.id(), state.id);
        assert_eq!(org.name(), state.name);
        assert_eq!(org.slug(), state.slug);
        assert_eq!(*org.created_at(), state.created_at);
        assert_eq!(org.version(), state.version);
    }
}
