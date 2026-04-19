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
    async fn list(&self) -> Result<Vec<Tag>, DomainError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Tag>, DomainError>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Tag>, DomainError>;
    async fn create(&self, name: &str) -> Result<Tag, DomainError>;
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
}

#[async_trait]
pub trait ResourceTagRepository: Send + Sync {
    async fn tags_for_resource(&self, resource_id: &str) -> Result<Vec<Tag>, DomainError>;
    async fn attach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError>;
    async fn detach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError>;
    /// Returns resource_ids that have the specified tag (by tag name).
    async fn resource_ids_with_tag(&self, tag_name: &str) -> Result<Vec<String>, DomainError>;
}
