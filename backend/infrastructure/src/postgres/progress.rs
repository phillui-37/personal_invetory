use async_trait::async_trait;
use domain::progress::{ProgressRepository, ResourceProgress};
use domain::DomainError;

#[cfg(feature = "postgres")]
use {
    crate::postgres::error::pg_err,
    chrono::{DateTime, Utc},
    sqlx::{PgPool, Row},
};

#[cfg(feature = "postgres")]
fn parse_ts(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| DomainError::InternalError(format!("timestamp parse: {e}")))
}

#[cfg(feature = "postgres")]
pub struct PgProgressRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgProgressRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl ProgressRepository for PgProgressRepository {
    async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError> {
        let row = sqlx::query(
            "SELECT resource_id, progress, notes, updated_at
             FROM resource_progress WHERE resource_id = $1",
        )
        .bind(resource_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?;
        row.map(|r| {
            let updated_at_raw: String = r.try_get("updated_at").map_err(pg_err)?;
            Ok(ResourceProgress {
                resource_id: r.try_get("resource_id").map_err(pg_err)?,
                progress: r.try_get("progress").map_err(pg_err)?,
                notes: r.try_get("notes").map_err(pg_err)?,
                updated_at: parse_ts(&updated_at_raw)?,
            })
        })
        .transpose()
    }

    async fn upsert(
        &self,
        resource_id: &str,
        progress: f64,
        notes: Option<&str>,
    ) -> Result<ResourceProgress, DomainError> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        sqlx::query(
            "INSERT INTO resource_progress (resource_id, progress, notes, updated_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (resource_id) DO UPDATE SET progress = $2, notes = $3, updated_at = $4",
        )
        .bind(resource_id)
        .bind(progress)
        .bind(notes)
        .bind(&now_str)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(ResourceProgress {
            resource_id: resource_id.to_string(),
            progress,
            notes: notes.map(str::to_string),
            updated_at: now,
        })
    }
}

#[cfg(not(feature = "postgres"))]
pub fn _placeholder() {}

