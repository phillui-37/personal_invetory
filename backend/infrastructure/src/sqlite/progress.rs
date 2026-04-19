use async_trait::async_trait;
use chrono::Utc;
use rusqlite::params;

use domain::progress::{ProgressRepository, ResourceProgress};
use domain::DomainError;

use super::SharedSqliteConnection;

pub struct SqliteProgressRepository {
    conn: SharedSqliteConnection,
}

impl SqliteProgressRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl ProgressRepository for SqliteProgressRepository {
    async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT resource_id, progress, notes, updated_at
                 FROM resource_progress
                 WHERE resource_id = ?1",
            )
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![resource_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        match rows.next() {
            None => Ok(None),
            Some(Err(e)) => Err(DomainError::InternalError(e.to_string())),
            Some(Ok((rid, progress, notes, updated_at_str))) => {
                let updated_at = updated_at_str
                    .parse::<chrono::DateTime<Utc>>()
                    .map_err(|e| DomainError::InternalError(e.to_string()))?;
                Ok(Some(ResourceProgress {
                    resource_id: rid,
                    progress,
                    notes,
                    updated_at,
                }))
            }
        }
    }

    async fn upsert(
        &self,
        resource_id: &str,
        progress: f64,
        notes: Option<&str>,
    ) -> Result<ResourceProgress, DomainError> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO resource_progress (resource_id, progress, notes, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![resource_id, progress, notes, now_str],
        )
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(ResourceProgress {
            resource_id: resource_id.to_string(),
            progress,
            notes: notes.map(|s| s.to_string()),
            updated_at: now,
        })
    }
}
