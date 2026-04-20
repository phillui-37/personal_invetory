use async_trait::async_trait;
use domain::{DomainError, Notification, NotificationRepository};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    crate::postgres::error::pg_err,
    chrono::Utc,
    sqlx::{PgPool, Row},
};

#[cfg(feature = "postgres")]
pub struct PgNotificationRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl NotificationRepository for PgNotificationRepository {
    async fn create(&self, resource_id: Uuid, message: String) -> Result<Notification, DomainError> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let created_at_str = created_at.to_rfc3339();
        sqlx::query(
            "INSERT INTO notifications (id, resource_id, message, created_at, read) \
             VALUES ($1, $2, $3, $4, 0)",
        )
        .bind(id.to_string())
        .bind(resource_id.to_string())
        .bind(&message)
        .bind(&created_at_str)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(Notification {
            id,
            resource_id,
            message,
            created_at,
            read: false,
        })
    }

    async fn list(&self, unread_only: bool) -> Result<Vec<Notification>, DomainError> {
        let rows = if unread_only {
            sqlx::query(
                "SELECT id, resource_id, message, created_at, read \
                 FROM notifications WHERE read = 0 ORDER BY created_at DESC",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(pg_err)?
        } else {
            sqlx::query(
                "SELECT id, resource_id, message, created_at, read \
                 FROM notifications ORDER BY created_at DESC",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(pg_err)?
        };

        rows.iter()
            .map(|row| {
                let id_s: String = row.try_get("id").map_err(pg_err)?;
                let id = Uuid::parse_str(&id_s)
                    .map_err(|e| DomainError::InternalError(e.to_string()))?;
                let rid_s: String = row.try_get("resource_id").map_err(pg_err)?;
                let resource_id = Uuid::parse_str(&rid_s)
                    .map_err(|e| DomainError::InternalError(e.to_string()))?;
                let ts: String = row.try_get("created_at").map_err(pg_err)?;
                let created_at = chrono::DateTime::parse_from_rfc3339(&ts)
                    .map(|d| d.with_timezone(&Utc))
                    .map_err(|e| DomainError::InternalError(format!("invalid timestamp: {e}")))?;
                let read_val: i64 = row.try_get("read").map_err(pg_err)?;
                Ok(Notification {
                    id,
                    resource_id,
                    message: row.try_get("message").map_err(pg_err)?,
                    created_at,
                    read: read_val != 0,
                })
            })
            .collect()
    }

    async fn mark_read(&self, id: Uuid) -> Result<(), DomainError> {
        let affected = sqlx::query("UPDATE notifications SET read = 1 WHERE id = $1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(pg_err)?
            .rows_affected();

        if affected == 0 {
            return Err(DomainError::NotFound(format!("notification {id} not found")));
        }
        Ok(())
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgNotificationRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl NotificationRepository for PgNotificationRepository {
    async fn create(&self, _resource_id: Uuid, _message: String) -> Result<Notification, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }

    async fn list(&self, _unread_only: bool) -> Result<Vec<Notification>, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }

    async fn mark_read(&self, _id: Uuid) -> Result<(), DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
