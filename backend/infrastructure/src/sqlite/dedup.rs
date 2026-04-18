use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use domain::dedup::{DedupWarning, DedupWarningRepository, DedupWarningStatus, NewDedupWarning};
use domain::DomainError;

use super::{map_sqlite_error, parse_timestamp_for_row, parse_uuid_for_row, SharedSqliteConnection};

pub struct SqliteDedupWarningRepository {
    conn: SharedSqliteConnection,
}

impl SqliteDedupWarningRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

fn decode_dedup_status(s: &str) -> DedupWarningStatus {
    match s {
        "dismissed" => DedupWarningStatus::Dismissed,
        "merged" => DedupWarningStatus::Merged,
        _ => DedupWarningStatus::Pending,
    }
}

#[async_trait]
impl DedupWarningRepository for SqliteDedupWarningRepository {
    async fn create(&self, warning: NewDedupWarning) -> Result<DedupWarning, DomainError> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4();
        let now = Utc::now();
        let id_str = id.to_string();
        // Normalize pair ordering to prevent (a,b)/(b,a) duplicates
        let (norm_a, norm_b) = if warning.resource_id_a <= warning.resource_id_b {
            (warning.resource_id_a, warning.resource_id_b)
        } else {
            (warning.resource_id_b, warning.resource_id_a)
        };
        let a_str = norm_a.to_string();
        let b_str = norm_b.to_string();
        let created_at_str = now.to_rfc3339();
        conn.execute(
            "INSERT INTO dedup_warnings (id, resource_id_a, resource_id_b, similarity_score, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
            rusqlite::params![id_str, a_str, b_str, warning.similarity_score, created_at_str],
        )
        .map_err(map_sqlite_error)?;
        Ok(DedupWarning {
            id,
            resource_id_a: norm_a,
            resource_id_b: norm_b,
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
                    id: parse_uuid_for_row(&id_str)?,
                    resource_id_a: parse_uuid_for_row(&a_str)?,
                    resource_id_b: parse_uuid_for_row(&b_str)?,
                    similarity_score: row.get(3)?,
                    status: DedupWarningStatus::Pending,
                    created_at: parse_timestamp_for_row(created_at_str)?,
                    resolved_at: None,
                })
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(warnings)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<DedupWarning, DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT resource_id_a, resource_id_b, similarity_score, status, created_at, resolved_at
             FROM dedup_warnings WHERE id = ?1",
            rusqlite::params![id.to_string()],
            |row| {
                let a_str: String = row.get(0)?;
                let b_str: String = row.get(1)?;
                let status_str: String = row.get(3)?;
                let created_at_str: String = row.get(4)?;
                let resolved_at_str: Option<String> = row.get(5)?;
                Ok(DedupWarning {
                    id,
                    resource_id_a: parse_uuid_for_row(&a_str)?,
                    resource_id_b: parse_uuid_for_row(&b_str)?,
                    similarity_score: row.get(2)?,
                    status: decode_dedup_status(&status_str),
                    created_at: parse_timestamp_for_row(created_at_str)?,
                    resolved_at: resolved_at_str
                        .map(parse_timestamp_for_row)
                        .transpose()?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DomainError::NotFound(format!("dedup warning {id} not found"))
            }
            other => map_sqlite_error(other),
        })
    }

    async fn dismiss(&self, id: Uuid) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let affected = conn
            .execute(
                "UPDATE dedup_warnings SET status = 'dismissed', resolved_at = ?1 WHERE id = ?2",
                rusqlite::params![now, id.to_string()],
            )
            .map_err(map_sqlite_error)?;
        if affected == 0 {
            return Err(DomainError::NotFound(format!(
                "dedup warning {id} not found"
            )));
        }
        Ok(())
    }

    async fn mark_merged(&self, id: Uuid) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let affected = conn
            .execute(
                "UPDATE dedup_warnings SET status = 'merged', resolved_at = ?1 WHERE id = ?2",
                rusqlite::params![now, id.to_string()],
            )
            .map_err(map_sqlite_error)?;
        if affected == 0 {
            return Err(DomainError::NotFound(format!(
                "dedup warning {id} not found"
            )));
        }
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
