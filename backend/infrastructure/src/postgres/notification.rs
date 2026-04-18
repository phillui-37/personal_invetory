use async_trait::async_trait;
use domain::{DomainError, Notification, NotificationRepository};
use uuid::Uuid;

#[derive(Default)]
pub struct PostgresNotificationRepository;

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn create(
        &self,
        _resource_id: Uuid,
        _message: String,
    ) -> Result<Notification, DomainError> {
        Err(DomainError::InternalError(
            "postgres notification repository is not implemented yet".to_string(),
        ))
    }

    async fn list(&self, _unread_only: bool) -> Result<Vec<Notification>, DomainError> {
        Err(DomainError::InternalError(
            "postgres notification repository is not implemented yet".to_string(),
        ))
    }

    async fn mark_read(&self, _id: Uuid) -> Result<(), DomainError> {
        Err(DomainError::InternalError(
            "postgres notification repository is not implemented yet".to_string(),
        ))
    }
}
