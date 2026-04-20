use async_trait::async_trait;
use domain::dedup::{DedupWarning, DedupWarningRepository, DedupWarningStatus, NewDedupWarning};
use domain::DomainError;
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    crate::postgres::error::pg_err,
    chrono::{DateTime, Utc},
    sqlx::{PgPool, Row},
};

#[cfg(feature = "postgres")]
fn decode_dedup_status(s: &str) -> DedupWarningStatus {
    match s {
        "dismissed" => DedupWarningStatus::Dismissed,
        "merged" => DedupWarningStatus::Merged,
        _ => DedupWarningStatus::Pending,
    }
}

#[cfg(feature = "postgres")]
fn parse_ts(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| DomainError::InternalError(format!("timestamp parse: {e}")))
}

#[cfg(feature = "postgres")]
pub struct PgDedupWarningRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgDedupWarningRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl DedupWarningRepository for PgDedupWarningRepository {
    async fn create(&self, warning: NewDedupWarning) -> Result<DedupWarning, DomainError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let (norm_a, norm_b) = if warning.resource_id_a <= warning.resource_id_b {
            (warning.resource_id_a, warning.resource_id_b)
        } else {
            (warning.resource_id_b, warning.resource_id_a)
        };
        let id_str = id.to_string();
        let a_str = norm_a.to_string();
        let b_str = norm_b.to_string();
        let now_str = now.to_rfc3339();
        sqlx::query(
            "INSERT INTO dedup_warnings (id, resource_id_a, resource_id_b, similarity_score, status, created_at)
             VALUES ($1, $2, $3, $4, 'pending', $5)",
        )
        .bind(&id_str)
        .bind(&a_str)
        .bind(&b_str)
        .bind(warning.similarity_score)
        .bind(&now_str)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
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
        let rows = sqlx::query(
            "SELECT id, resource_id_a, resource_id_b, similarity_score, created_at
             FROM dedup_warnings WHERE status = 'pending' ORDER BY similarity_score DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;
        rows.iter()
            .map(|r| {
                let id: Uuid = Uuid::parse_str(r.try_get::<String, _>("id").map_err(pg_err)?.as_str())
                    .map_err(|e| DomainError::InternalError(format!("uuid: {e}")))?;
                let a: Uuid = Uuid::parse_str(r.try_get::<String, _>("resource_id_a").map_err(pg_err)?.as_str())
                    .map_err(|e| DomainError::InternalError(format!("uuid: {e}")))?;
                let b: Uuid = Uuid::parse_str(r.try_get::<String, _>("resource_id_b").map_err(pg_err)?.as_str())
                    .map_err(|e| DomainError::InternalError(format!("uuid: {e}")))?;
                let created_at_raw: String = r.try_get("created_at").map_err(pg_err)?;
                Ok(DedupWarning {
                    id,
                    resource_id_a: a,
                    resource_id_b: b,
                    similarity_score: r.try_get("similarity_score").map_err(pg_err)?,
                    status: DedupWarningStatus::Pending,
                    created_at: parse_ts(&created_at_raw)?,
                    resolved_at: None,
                })
            })
            .collect()
    }

    async fn get_by_id(&self, id: Uuid) -> Result<DedupWarning, DomainError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT resource_id_a, resource_id_b, similarity_score, status, created_at, resolved_at
             FROM dedup_warnings WHERE id = $1",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| DomainError::NotFound(format!("dedup warning {id} not found")))?;
        let a: Uuid = Uuid::parse_str(row.try_get::<String, _>("resource_id_a").map_err(pg_err)?.as_str())
            .map_err(|e| DomainError::InternalError(format!("uuid: {e}")))?;
        let b: Uuid = Uuid::parse_str(row.try_get::<String, _>("resource_id_b").map_err(pg_err)?.as_str())
            .map_err(|e| DomainError::InternalError(format!("uuid: {e}")))?;
        let status_str: String = row.try_get("status").map_err(pg_err)?;
        let created_at_raw: String = row.try_get("created_at").map_err(pg_err)?;
        let resolved_at_raw: Option<String> = row.try_get("resolved_at").map_err(pg_err)?;
        Ok(DedupWarning {
            id,
            resource_id_a: a,
            resource_id_b: b,
            similarity_score: row.try_get("similarity_score").map_err(pg_err)?,
            status: decode_dedup_status(&status_str),
            created_at: parse_ts(&created_at_raw)?,
            resolved_at: resolved_at_raw.map(|s| parse_ts(&s)).transpose()?,
        })
    }

    async fn dismiss(&self, id: Uuid) -> Result<(), DomainError> {
        let id_str = id.to_string();
        let now_str = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE dedup_warnings SET status = 'dismissed', resolved_at = $1 WHERE id = $2",
        )
        .bind(&now_str)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("dedup warning {id} not found")));
        }
        Ok(())
    }

    async fn mark_merged(&self, id: Uuid) -> Result<(), DomainError> {
        let id_str = id.to_string();
        let now_str = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE dedup_warnings SET status = 'merged', resolved_at = $1 WHERE id = $2",
        )
        .bind(&now_str)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("dedup warning {id} not found")));
        }
        Ok(())
    }

    async fn exists_pair(&self, a: Uuid, b: Uuid) -> Result<bool, DomainError> {
        let a_str = a.to_string();
        let b_str = b.to_string();
        let row = sqlx::query(
            "SELECT COUNT(*) AS cnt FROM dedup_warnings
             WHERE ((resource_id_a = $1 AND resource_id_b = $2)
                 OR (resource_id_a = $2 AND resource_id_b = $1))
               AND status = 'pending'",
        )
        .bind(&a_str)
        .bind(&b_str)
        .fetch_one(&self.pool)
        .await
        .map_err(pg_err)?;
        let count: i64 = row.try_get("cnt").map_err(pg_err)?;
        Ok(count > 0)
    }
}

#[cfg(not(feature = "postgres"))]
pub fn _placeholder() {}

