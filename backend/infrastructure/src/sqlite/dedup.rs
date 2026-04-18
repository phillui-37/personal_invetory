use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use domain::dedup::{DedupWarning, DedupWarningRepository, DedupWarningStatus, NewDedupWarning};
use domain::DomainError;

use super::{map_sqlite_error, parse_timestamp, SharedSqliteConnection};

pub struct SqliteDedupWarningRepository {
    conn: SharedSqliteConnection,
}

impl SqliteDedupWarningRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl DedupWarningRepository for SqliteDedupWarningRepository {
    async fn create(&self, warning: NewDedupWarning) -> Result<DedupWarning, DomainError> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4();
        let now = Utc::now();
        let id_str = id.to_string();
        let a_str = warning.resource_id_a.to_string();
        let b_str = warning.resource_id_b.to_string();
        let created_at_str = now.to_rfc3339();
        conn.execute(
            "INSERT INTO dedup_warnings (id, resource_id_a, resource_id_b, similarity_score, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
            rusqlite::params![id_str, a_str, b_str, warning.similarity_score, created_at_str],
        )
        .map_err(map_sqlite_error)?;
        Ok(DedupWarning {
            id,
            resource_id_a: warning.resource_id_a,
            resource_id_b: warning.resource_id_b,
            similarity_score: warning.similarity_score,
            status: DedupWarningStatus::Pending,
            created_at: now,
            resolved_at: None,
        })
    }

    async fn list_pending(&self) -> Result<Vec<DedupWarning>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, resource_id_a, resource_id_b, similarity_score, created_at
                 FROM dedup_warnings WHERE status = 'pending' ORDER BY similarity_score DESC",
            )
            .map_err(map_sqlite_error)?;
        let warnings = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let a_str: String = row.get(1)?;
                let b_str: String = row.get(2)?;
                let created_at_str: String = row.get(4)?;
                Ok(DedupWarning {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    resource_id_a: Uuid::parse_str(&a_str).unwrap_or_default(),
                    resource_id_b: Uuid::parse_str(&b_str).unwrap_or_default(),
                    similarity_score: row.get(3)?,
                    status: DedupWarningStatus::Pending,
                    created_at: parse_timestamp(created_at_str).unwrap_or_default(),
                    resolved_at: None,
                })
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(warnings)
    }

    async fn dismiss(&self, id: Uuid) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE dedup_warnings SET status = 'dismissed', resolved_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id.to_string()],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn mark_merged(&self, id: Uuid) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE dedup_warnings SET status = 'merged', resolved_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id.to_string()],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn exists_pair(&self, a: Uuid, b: Uuid) -> Result<bool, DomainError> {
        let conn = self.conn.lock().unwrap();
        let a_str = a.to_string();
        let b_str = b.to_string();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dedup_warnings
                 WHERE (resource_id_a = ?1 AND resource_id_b = ?2)
                    OR (resource_id_a = ?2 AND resource_id_b = ?1)",
                rusqlite::params![a_str, b_str],
                |row| row.get(0),
            )
            .map_err(map_sqlite_error)?;
        Ok(count > 0)
    }
}
