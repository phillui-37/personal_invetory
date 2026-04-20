use async_trait::async_trait;
use chrono::Utc;
use rusqlite::{params, OptionalExtension};

use domain::progress::{ProgressRepository, ResourceProgress};
use domain::DomainError;

use crate::sqlite::{map_sqlite_error, parse_timestamp_for_row, SharedSqliteConnection};

pub struct SqliteProgressRepository {
    conn: SharedSqliteConnection,
}

impl SqliteProgressRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }

    fn row_to_progress(row: &rusqlite::Row<'_>) -> rusqlite::Result<ResourceProgress> {
        let resource_id: String = row.get(0)?;
        let progress: f64 = row.get(1)?;
        let notes: Option<String> = row.get(2)?;
        let updated_at_raw: String = row.get(3)?;
        let updated_at = parse_timestamp_for_row(updated_at_raw)?;
        Ok(ResourceProgress {
            resource_id,
            progress,
            notes,
            updated_at,
        })
    }
}

#[async_trait]
impl ProgressRepository for SqliteProgressRepository {
    async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        conn.query_row(
            "SELECT resource_id, progress, notes, updated_at
             FROM resource_progress
             WHERE resource_id = ?1",
            params![resource_id],
            Self::row_to_progress,
        )
        .optional()
        .map_err(map_sqlite_error)
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
        .map_err(map_sqlite_error)?;

        Ok(ResourceProgress {
            resource_id: resource_id.to_string(),
            progress,
            notes: notes.map(|s| s.to_string()),
            updated_at: now,
        })
    }
}
