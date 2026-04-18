use crate::sqlite::{map_sqlite_error, parse_timestamp, SharedSqliteConnection};
use async_trait::async_trait;
use chrono::Utc;
use domain::{DomainError, Notification, NotificationRepository};
use rusqlite::params;
use uuid::Uuid;

pub struct SqliteNotificationRepository {
    conn: SharedSqliteConnection,
}

impl SqliteNotificationRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl NotificationRepository for SqliteNotificationRepository {
    async fn create(
        &self,
        resource_id: Uuid,
        message: String,
    ) -> Result<Notification, DomainError> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO notifications (id, resource_id, message, created_at, read)
             VALUES (?1, ?2, ?3, ?4, 0)",
            params![
                id.to_string(),
                resource_id.to_string(),
                message.as_str(),
                created_at.to_rfc3339()
            ],
        )
        .map_err(map_sqlite_error)?;

        Ok(Notification {
            id,
            resource_id,
            message,
            created_at,
            read: false,
        })
    }

    async fn list(&self, unread_only: bool) -> Result<Vec<Notification>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        let sql = if unread_only {
            "SELECT id, resource_id, message, created_at, read FROM notifications WHERE read = 0 ORDER BY created_at DESC"
        } else {
            "SELECT id, resource_id, message, created_at, read FROM notifications ORDER BY created_at DESC"
        };

        let mut stmt = conn.prepare(sql).map_err(map_sqlite_error)?;

        let notifications = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let resource_id_str: String = row.get(1)?;
                let message: String = row.get(2)?;
                let created_at_str: String = row.get(3)?;
                let read: i32 = row.get(4)?;
                Ok((id_str, resource_id_str, message, created_at_str, read))
            })
            .map_err(map_sqlite_error)?
            .map(|row| {
                let (id_str, resource_id_str, message, created_at_str, read) =
                    row.map_err(map_sqlite_error)?;
                let id = Uuid::parse_str(&id_str)
                    .map_err(|e| DomainError::InternalError(format!("invalid uuid: {e}")))?;
                let resource_id = Uuid::parse_str(&resource_id_str)
                    .map_err(|e| DomainError::InternalError(format!("invalid uuid: {e}")))?;
                let created_at = parse_timestamp(created_at_str)?;
                Ok(Notification {
                    id,
                    resource_id,
                    message,
                    created_at,
                    read: read != 0,
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;

        Ok(notifications)
    }

    async fn mark_read(&self, id: Uuid) -> Result<(), DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE notifications SET read = 1 WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(map_sqlite_error)?;

        if rows_affected == 0 {
            return Err(DomainError::NotFound(format!("notification {id} not found")));
        }
        Ok(())
    }
}
