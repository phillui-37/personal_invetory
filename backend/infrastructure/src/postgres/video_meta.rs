use async_trait::async_trait;
use domain::{DomainError, VideoMeta, VideoMetaRepository, NewVideoMeta};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {crate::postgres::error::pg_err, sqlx::{PgPool, Row}};

#[cfg(feature = "postgres")]
pub struct PgVideoMetaRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgVideoMetaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl VideoMetaRepository for PgVideoMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<VideoMeta, DomainError> {
        let row = sqlx::query(
            "SELECT resource_id, duration_secs, file_format, resolution, file_size_bytes \
             FROM video_metas WHERE resource_id = $1",
        )
        .bind(resource_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| {
            DomainError::NotFound(format!("video meta for resource {resource_id} not found"))
        })?;

        let rid: String = row.try_get("resource_id").map_err(pg_err)?;
        let duration_secs: Option<i64> = row.try_get("duration_secs").map_err(pg_err)?;
        let file_size_bytes: Option<i64> = row.try_get("file_size_bytes").map_err(pg_err)?;
        Ok(VideoMeta {
            resource_id: Uuid::parse_str(&rid)
                .map_err(|e| DomainError::InternalError(e.to_string()))?,
            duration_secs: duration_secs.map(|v| u64::try_from(v).unwrap_or(0)),
            file_format: row.try_get("file_format").map_err(pg_err)?,
            resolution: row.try_get("resolution").map_err(pg_err)?,
            file_size_bytes: file_size_bytes.map(|v| u64::try_from(v).unwrap_or(0)),
        })
    }

    async fn upsert(&self, resource_id: Uuid, input: NewVideoMeta) -> Result<VideoMeta, DomainError> {
        let duration_secs = input.duration_secs.map(|v| v as i64);
        let file_size_bytes = input.file_size_bytes.map(|v| v as i64);
        sqlx::query(
            "INSERT INTO video_metas (resource_id, duration_secs, file_format, resolution, file_size_bytes) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT(resource_id) DO UPDATE SET \
               duration_secs = EXCLUDED.duration_secs, \
               file_format = EXCLUDED.file_format, \
               resolution = EXCLUDED.resolution, \
               file_size_bytes = EXCLUDED.file_size_bytes",
        )
        .bind(resource_id.to_string())
        .bind(duration_secs)
        .bind(&input.file_format)
        .bind(&input.resolution)
        .bind(file_size_bytes)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(VideoMeta {
            resource_id,
            duration_secs: input.duration_secs,
            file_format: input.file_format,
            resolution: input.resolution,
            file_size_bytes: input.file_size_bytes,
        })
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgVideoMetaRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl VideoMetaRepository for PgVideoMetaRepository {
    async fn get(&self, _resource_id: Uuid) -> Result<VideoMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn upsert(&self, _resource_id: Uuid, _input: NewVideoMeta) -> Result<VideoMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
