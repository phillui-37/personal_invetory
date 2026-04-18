use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SyncJob {
    pub id: Uuid,
    pub platform: String,
    pub status: SyncJobStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub items_found: u32,
    pub items_created: u32,
    pub items_skipped: u32,
    pub items_failed: u32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct NewSyncJob {
    pub platform: String,
}

#[async_trait]
pub trait SyncJobRepository: Send + Sync {
    async fn create(&self, job: NewSyncJob) -> Result<SyncJob, DomainError>;
    async fn update(&self, job: &SyncJob) -> Result<(), DomainError>;
    async fn get(&self, id: Uuid) -> Result<SyncJob, DomainError>;
    async fn list_by_platform(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_job_status_equality() {
        assert_eq!(SyncJobStatus::Pending, SyncJobStatus::Pending);
        assert_ne!(SyncJobStatus::Pending, SyncJobStatus::Running);
    }

    #[test]
    fn new_sync_job_holds_platform() {
        let job = NewSyncJob {
            platform: "steam".to_string(),
        };
        assert_eq!(job.platform, "steam");
    }
}
