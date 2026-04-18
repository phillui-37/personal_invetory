use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use domain::sync::{NewSyncJob, SyncJob, SyncJobRepository, SyncJobStatus};
use domain::DomainError;

use super::{map_sqlite_error, parse_timestamp, SharedSqliteConnection};

pub struct SqliteSyncJobRepository {
    conn: SharedSqliteConnection,
}

impl SqliteSyncJobRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

fn encode_sync_status(status: &SyncJobStatus) -> &'static str {
    match status {
        SyncJobStatus::Pending => "pending",
        SyncJobStatus::Running => "running",
        SyncJobStatus::Completed => "completed",
        SyncJobStatus::Failed => "failed",
    }
}

fn decode_sync_status(s: &str) -> SyncJobStatus {
    match s {
        "running" => SyncJobStatus::Running,
        "completed" => SyncJobStatus::Completed,
        "failed" => SyncJobStatus::Failed,
        _ => SyncJobStatus::Pending,
    }
}

#[async_trait]
impl SyncJobRepository for SqliteSyncJobRepository {
    async fn create(&self, job: NewSyncJob) -> Result<SyncJob, DomainError> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4();
        let now = Utc::now();
        let id_str = id.to_string();
        let created_at_str = now.to_rfc3339();
        conn.execute(
            "INSERT INTO sync_jobs (id, platform, status, created_at) VALUES (?1, ?2, 'pending', ?3)",
            rusqlite::params![id_str, job.platform, created_at_str],
        )
        .map_err(map_sqlite_error)?;
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
        let conn = self.conn.lock().unwrap();
        let id_str = job.id.to_string();
        let status_str = encode_sync_status(&job.status);
        let started_at = job.started_at.map(|t| t.to_rfc3339());
        let completed_at = job.completed_at.map(|t| t.to_rfc3339());
        conn.execute(
            "UPDATE sync_jobs SET status = ?1, started_at = ?2, completed_at = ?3,
             items_found = ?4, items_created = ?5, items_skipped = ?6, items_failed = ?7,
             error_message = ?8 WHERE id = ?9",
            rusqlite::params![
                status_str, started_at, completed_at,
                job.items_found, job.items_created, job.items_skipped, job.items_failed,
                job.error_message, id_str
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<SyncJob, DomainError> {
        let conn = self.conn.lock().unwrap();
        let id_str = id.to_string();
        conn.query_row(
            "SELECT platform, status, started_at, completed_at, items_found, items_created,
             items_skipped, items_failed, error_message, created_at FROM sync_jobs WHERE id = ?1",
            rusqlite::params![id_str],
            |row| {
                let status_str: String = row.get(1)?;
                let started_at: Option<String> = row.get(2)?;
                let completed_at: Option<String> = row.get(3)?;
                let created_at_str: String = row.get(9)?;
                Ok(SyncJob {
                    id,
                    platform: row.get(0)?,
                    status: decode_sync_status(&status_str),
                    started_at: started_at.map(|s| parse_timestamp(s).unwrap_or_default()),
                    completed_at: completed_at.map(|s| parse_timestamp(s).unwrap_or_default()),
                    items_found: u32::try_from(row.get::<_, i64>(4)?).unwrap_or(0),
                    items_created: u32::try_from(row.get::<_, i64>(5)?).unwrap_or(0),
                    items_skipped: u32::try_from(row.get::<_, i64>(6)?).unwrap_or(0),
                    items_failed: u32::try_from(row.get::<_, i64>(7)?).unwrap_or(0),
                    error_message: row.get(8)?,
                    created_at: parse_timestamp(created_at_str).unwrap_or_default(),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DomainError::NotFound(format!("sync job {id} not found"))
            }
            other => map_sqlite_error(other),
        })
    }

    async fn list_by_platform(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, status, started_at, completed_at, items_found, items_created,
                 items_skipped, items_failed, error_message, created_at
                 FROM sync_jobs WHERE platform = ?1 ORDER BY created_at DESC",
            )
            .map_err(map_sqlite_error)?;
        let jobs = stmt
            .query_map(rusqlite::params![platform], |row| {
                let id_str: String = row.get(0)?;
                let status_str: String = row.get(1)?;
                let started_at: Option<String> = row.get(2)?;
                let completed_at: Option<String> = row.get(3)?;
                let created_at_str: String = row.get(9)?;
                Ok(SyncJob {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    platform: platform.to_string(),
                    status: decode_sync_status(&status_str),
                    started_at: started_at.map(|s| parse_timestamp(s).unwrap_or_default()),
                    completed_at: completed_at.map(|s| parse_timestamp(s).unwrap_or_default()),
                    items_found: u32::try_from(row.get::<_, i64>(4)?).unwrap_or(0),
                    items_created: u32::try_from(row.get::<_, i64>(5)?).unwrap_or(0),
                    items_skipped: u32::try_from(row.get::<_, i64>(6)?).unwrap_or(0),
                    items_failed: u32::try_from(row.get::<_, i64>(7)?).unwrap_or(0),
                    error_message: row.get(8)?,
                    created_at: parse_timestamp(created_at_str).unwrap_or_default(),
                })
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(jobs)
    }
}
