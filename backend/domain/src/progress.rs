use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::DomainError;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceProgress {
    pub resource_id: String,
    pub progress: f64,        // 0.0 – 1.0
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait ProgressRepository: Send + Sync {
    /// Returns None if no progress row exists for this resource_id.
    async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError>;

    /// Upsert — inserts or replaces the progress row.
    async fn upsert(
        &self,
        resource_id: &str,
        progress: f64,
        notes: Option<&str>,
    ) -> Result<ResourceProgress, DomainError>;
}
