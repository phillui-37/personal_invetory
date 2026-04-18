use crate::sqlite::{map_sqlite_error, SharedSqliteConnection};
use async_trait::async_trait;
use domain::{DomainError, NewVideoMeta, VideoMeta, VideoMetaRepository};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

pub struct SqliteVideoMetaRepository {
    conn: SharedSqliteConnection,
}

impl SqliteVideoMetaRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl VideoMetaRepository for SqliteVideoMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<VideoMeta, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.query_row(
            "SELECT duration_secs, file_format, resolution, file_size_bytes
             FROM video_metas
             WHERE resource_id = ?1",
            params![resource_id.to_string()],
            |row| {
                Ok(VideoMeta {
                    resource_id,
                    duration_secs: row.get::<_, Option<i64>>(0)?.map(|v| v as u64),
                    file_format: row.get(1)?,
                    resolution: row.get(2)?,
                    file_size_bytes: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                })
            },
        )
        .optional()
        .map_err(map_sqlite_error)?
        .ok_or_else(|| DomainError::NotFound(format!("video meta {resource_id} not found")))
    }

    async fn upsert(&self, resource_id: Uuid, input: NewVideoMeta) -> Result<VideoMeta, DomainError> {
        let duration_secs = input.duration_secs.map(|v| v as i64);
        let file_format = input.file_format;
        let resolution = input.resolution;
        let file_size_bytes = input.file_size_bytes.map(|v| v as i64);
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO video_metas (resource_id, duration_secs, file_format, resolution, file_size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(resource_id) DO UPDATE SET
                 duration_secs = excluded.duration_secs,
                 file_format = excluded.file_format,
                 resolution = excluded.resolution,
                 file_size_bytes = excluded.file_size_bytes",
            params![
                resource_id.to_string(),
                duration_secs,
                file_format.as_deref(),
                resolution.as_deref(),
                file_size_bytes
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(VideoMeta {
            resource_id,
            duration_secs: input.duration_secs,
            file_format,
            resolution,
            file_size_bytes: input.file_size_bytes,
        })
    }
}
