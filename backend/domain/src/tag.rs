use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::DomainError;

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait TagRepository: Send + Sync {
    /// Returns all tags, ordered by name.
    async fn list(&self) -> Result<Vec<Tag>, DomainError>;

    /// Returns `None` if no tag exists with `id`.
    async fn get_by_id(&self, id: &str) -> Result<Option<Tag>, DomainError>;

    /// Returns `None` if no tag exists with `name`.
    async fn get_by_name(&self, name: &str) -> Result<Option<Tag>, DomainError>;

    /// Creates a new tag. Returns [`DomainError::Conflict`] if `name` is already taken.
    async fn create(&self, name: &str) -> Result<Tag, DomainError>;

    /// Deletes the tag and cascades removal of all resource_tags associations.
    /// Returns [`DomainError::NotFound`] if `id` is unknown.
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
}

#[async_trait]
pub trait ResourceTagRepository: Send + Sync {
    /// Returns all tags attached to `resource_id`.
    async fn tags_for_resource(&self, resource_id: &str) -> Result<Vec<Tag>, DomainError>;

    /// Idempotent — succeeds silently if the association already exists.
    async fn attach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError>;

    /// Returns [`DomainError::NotFound`] if the association does not exist.
    async fn detach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError>;

    /// Returns resource_ids that have the tag with `tag_id`.
    /// Name→ID resolution is done by the service layer (via `TagRepository::get_by_name`).
    async fn resource_ids_with_tag_id(&self, tag_id: &str) -> Result<Vec<String>, DomainError>;
}
