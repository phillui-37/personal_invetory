use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DedupWarningStatus {
    Pending,
    Dismissed,
    Merged,
}

#[derive(Debug, Clone)]
pub struct DedupWarning {
    pub id: Uuid,
    pub resource_id_a: Uuid,
    pub resource_id_b: Uuid,
    pub similarity_score: f64,
    pub status: DedupWarningStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

pub struct NewDedupWarning {
    pub resource_id_a: Uuid,
    pub resource_id_b: Uuid,
    pub similarity_score: f64,
}

#[async_trait]
pub trait DedupWarningRepository: Send + Sync {
    async fn create(&self, warning: NewDedupWarning) -> Result<DedupWarning, DomainError>;
    async fn list_pending(&self) -> Result<Vec<DedupWarning>, DomainError>;
    async fn dismiss(&self, id: Uuid) -> Result<(), DomainError>;
    async fn mark_merged(&self, id: Uuid) -> Result<(), DomainError>;
    async fn exists_pair(&self, a: Uuid, b: Uuid) -> Result<bool, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_warning_status_equality() {
        assert_eq!(DedupWarningStatus::Pending, DedupWarningStatus::Pending);
        assert_ne!(DedupWarningStatus::Pending, DedupWarningStatus::Dismissed);
    }

    #[test]
    fn new_dedup_warning_holds_fields() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let warning = NewDedupWarning {
            resource_id_a: a,
            resource_id_b: b,
            similarity_score: 0.92,
        };
        assert_eq!(warning.resource_id_a, a);
        assert_eq!(warning.resource_id_b, b);
        assert!((warning.similarity_score - 0.92).abs() < f64::EPSILON);
    }
}
