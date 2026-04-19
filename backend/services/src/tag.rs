use std::sync::Arc;

use domain::tag::{ResourceTagRepository, Tag, TagRepository};
use domain::DomainError;

pub struct TagService {
    tag_repo: Arc<dyn TagRepository>,
    resource_tag_repo: Arc<dyn ResourceTagRepository>,
}

impl TagService {
    pub fn new(
        tag_repo: Arc<dyn TagRepository>,
        resource_tag_repo: Arc<dyn ResourceTagRepository>,
    ) -> Self {
        Self {
            tag_repo,
            resource_tag_repo,
        }
    }

    fn normalize_name(name: &str) -> Result<String, DomainError> {
        let normalized = name.trim().to_lowercase();
        if normalized.is_empty() {
            return Err(DomainError::ValidationError(
                "tag name must not be empty".into(),
            ));
        }
        Ok(normalized)
    }

    pub async fn list(&self) -> Result<Vec<Tag>, DomainError> {
        self.tag_repo.list().await
    }

    pub async fn create(&self, name: &str) -> Result<Tag, DomainError> {
        let normalized = Self::normalize_name(name)?;
        self.tag_repo.create(&normalized).await
    }

    pub async fn delete(&self, id: &str) -> Result<(), DomainError> {
        self.tag_repo.delete(id).await
    }

    pub async fn tags_for_resource(&self, resource_id: &str) -> Result<Vec<Tag>, DomainError> {
        self.resource_tag_repo.tags_for_resource(resource_id).await
    }

    pub async fn attach_tag(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        self.resource_tag_repo.attach(resource_id, tag_id).await
    }

    pub async fn detach_tag(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        self.resource_tag_repo.detach(resource_id, tag_id).await
    }

    pub async fn resource_ids_for_tag_name(
        &self,
        tag_name: &str,
    ) -> Result<Vec<String>, DomainError> {
        let normalized = Self::normalize_name(tag_name)?;
        let Some(tag) = self.tag_repo.get_by_name(&normalized).await? else {
            return Ok(vec![]);
        };
        self.resource_tag_repo
            .resource_ids_with_tag_id(&tag.id)
            .await
    }
}
