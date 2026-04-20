use async_trait::async_trait;
use domain::sync::{NewSyncJob, SyncJob, SyncJobRepository, SyncJobStatus};
use domain::DomainError;
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    crate::postgres::error::pg_err,
    chrono::{DateTime, Utc},
    sqlx::{PgPool, Row},
};

#[cfg(feature = "postgres")]
fn encode_sync_status(status: &SyncJobStatus) -> &'static str {
    match status {
        SyncJobStatus::Pending => "pending",
        SyncJobStatus::Running => "running",
        SyncJobStatus::Completed => "completed",
        SyncJobStatus::Failed => "failed",
    }
}

#[cfg(feature = "postgres")]
fn decode_sync_status(s: &str) -> SyncJobStatus {
    match s {
        "running" => SyncJobStatus::Running,
        "completed" => SyncJobStatus::Completed,
        "failed" => SyncJobStatus::Failed,
        _ => SyncJobStatus::Pending,
    }
}

#[cfg(feature = "postgres")]
fn parse_ts(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| DomainError::InternalError(format!("timestamp parse: {e}")))
}

#[cfg(feature = "postgres")]
pub struct PgSyncJobRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgSyncJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl SyncJobRepository for PgSyncJobRepository {
    async fn create(&self, job: NewSyncJob) -> Result<SyncJob, DomainError> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        sqlx::query(
            "INSERT INTO sync_jobs (id, platform, status, created_at)
             VALUES ($1, $2, 'pending', $3)",
        )
        .bind(&id_str)
        .bind(&job.platform)
        .bind(&now_str)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(SyncJob {
            id,
            platform: job.platform,
            status: SyncJobStatus::Pending,
            started_at: None,
            completed_at: None,
            items_found: 0,
            items_created: 0,
            items_skipped: 0,
            items_failed: 0,
            error_message: None,
            created_at: now,
        })
    }

    async fn update(&self, job: &SyncJob) -> Result<(), DomainError> {
        let id_str = job.id.to_string();
        let status_str = encode_sync_status(&job.status);
        let started_at = job.started_at.map(|t| t.to_rfc3339());
        let completed_at = job.completed_at.map(|t| t.to_rfc3339());
        let result = sqlx::query(
            "UPDATE sync_jobs SET status = $1, started_at = $2, completed_at = $3,
             items_found = $4, items_created = $5, items_skipped = $6, items_failed = $7,
             error_message = $8 WHERE id = $9",
        )
        .bind(status_str)
        .bind(&started_at)
        .bind(&completed_at)
        .bind(job.items_found as i64)
        .bind(job.items_created as i64)
        .bind(job.items_skipped as i64)
        .bind(job.items_failed as i64)
        .bind(&job.error_message)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("sync job {} not found", job.id)));
        }
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<SyncJob, DomainError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT platform, status, started_at, completed_at, items_found, items_created,
             items_skipped, items_failed, error_message, created_at
             FROM sync_jobs WHERE id = $1",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| DomainError::NotFound(format!("sync job {id} not found")))?;

        let status_str: String = row.try_get("status").map_err(pg_err)?;
        let started_at_raw: Option<String> = row.try_get("started_at").map_err(pg_err)?;
        let completed_at_raw: Option<String> = row.try_get("completed_at").map_err(pg_err)?;
        let created_at_raw: String = row.try_get("created_at").map_err(pg_err)?;
        Ok(SyncJob {
            id,
            platform: row.try_get("platform").map_err(pg_err)?,
            status: decode_sync_status(&status_str),
            started_at: started_at_raw.map(|s| parse_ts(&s)).transpose()?,
            completed_at: completed_at_raw.map(|s| parse_ts(&s)).transpose()?,
            items_found: u32::try_from(row.try_get::<i64, _>("items_found").map_err(pg_err)?).unwrap_or(0),
            items_created: u32::try_from(row.try_get::<i64, _>("items_created").map_err(pg_err)?).unwrap_or(0),
            items_skipped: u32::try_from(row.try_get::<i64, _>("items_skipped").map_err(pg_err)?).unwrap_or(0),
            items_failed: u32::try_from(row.try_get::<i64, _>("items_failed").map_err(pg_err)?).unwrap_or(0),
            error_message: row.try_get("error_message").map_err(pg_err)?,
            created_at: parse_ts(&created_at_raw)?,
        })
    }

    async fn list_by_platform(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, status, started_at, completed_at, items_found, items_created,
             items_skipped, items_failed, error_message, created_at
             FROM sync_jobs WHERE platform = $1 ORDER BY created_at DESC",
        )
        .bind(platform)
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;

        rows.iter()
            .map(|r| {
                let id_str: String = r.try_get("id").map_err(pg_err)?;
                let id = Uuid::parse_str(&id_str)
                    .map_err(|e| DomainError::InternalError(format!("uuid parse: {e}")))?;
                let status_str: String = r.try_get("status").map_err(pg_err)?;
                let started_at_raw: Option<String> = r.try_get("started_at").map_err(pg_err)?;
                let completed_at_raw: Option<String> = r.try_get("completed_at").map_err(pg_err)?;
                let created_at_raw: String = r.try_get("created_at").map_err(pg_err)?;
                Ok(SyncJob {
                    id,
                    platform: platform.to_string(),
                    status: decode_sync_status(&status_str),
                    started_at: started_at_raw.map(|s| parse_ts(&s)).transpose()?,
                    completed_at: completed_at_raw.map(|s| parse_ts(&s)).transpose()?,
                    items_found: u32::try_from(r.try_get::<i64, _>("items_found").map_err(pg_err)?).unwrap_or(0),
                    items_created: u32::try_from(r.try_get::<i64, _>("items_created").map_err(pg_err)?).unwrap_or(0),
                    items_skipped: u32::try_from(r.try_get::<i64, _>("items_skipped").map_err(pg_err)?).unwrap_or(0),
                    items_failed: u32::try_from(r.try_get::<i64, _>("items_failed").map_err(pg_err)?).unwrap_or(0),
                    error_message: r.try_get("error_message").map_err(pg_err)?,
                    created_at: parse_ts(&created_at_raw)?,
                })
            })
            .collect()
    }
}

#[cfg(not(feature = "postgres"))]
pub fn _placeholder() {}

