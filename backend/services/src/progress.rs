use std::sync::Arc;

use domain::progress::{ProgressRepository, ResourceProgress};
use domain::DomainError;

pub struct ProgressService {
    repo: Arc<dyn ProgressRepository>,
}

impl ProgressService {
    pub fn new(repo: Arc<dyn ProgressRepository>) -> Self {
        Self { repo }
    }

    pub async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError> {
        self.repo.get(resource_id).await
    }

    pub async fn upsert(
        &self,
        resource_id: &str,
        progress: f64,
        notes: Option<&str>,
    ) -> Result<ResourceProgress, DomainError> {
        if !(0.0..=1.0).contains(&progress) {
            return Err(DomainError::ValidationError("progress must be 0.0–1.0".into()));
        }
        self.repo.upsert(resource_id, progress, notes).await
    }
}
