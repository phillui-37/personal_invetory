use async_trait::async_trait;
use domain::{DomainError, ImageMeta, ImageMetaRepository, NewImageMeta};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {crate::postgres::error::pg_err, sqlx::{PgPool, Row}};

#[cfg(feature = "postgres")]
pub struct PgImageMetaRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgImageMetaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl ImageMetaRepository for PgImageMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<ImageMeta, DomainError> {
        let row = sqlx::query(
            "SELECT resource_id, width, height, file_format, file_size_bytes \
             FROM image_metas WHERE resource_id = $1",
        )
        .bind(resource_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| {
            DomainError::NotFound(format!("image meta for resource {resource_id} not found"))
        })?;

        let rid: String = row.try_get("resource_id").map_err(pg_err)?;
        let width: Option<i64> = row.try_get("width").map_err(pg_err)?;
        let height: Option<i64> = row.try_get("height").map_err(pg_err)?;
        let file_size_bytes: Option<i64> = row.try_get("file_size_bytes").map_err(pg_err)?;
        Ok(ImageMeta {
            resource_id: Uuid::parse_str(&rid)
                .map_err(|e| DomainError::InternalError(e.to_string()))?,
            width: width.map(|v| u32::try_from(v).unwrap_or(0)),
            height: height.map(|v| u32::try_from(v).unwrap_or(0)),
            file_format: row.try_get("file_format").map_err(pg_err)?,
            file_size_bytes: file_size_bytes.map(|v| u64::try_from(v).unwrap_or(0)),
        })
    }

    async fn upsert(&self, resource_id: Uuid, input: NewImageMeta) -> Result<ImageMeta, DomainError> {
        let width = input.width.map(|v| v as i64);
        let height = input.height.map(|v| v as i64);
        let file_size_bytes = input.file_size_bytes.map(|v| v as i64);
        sqlx::query(
            "INSERT INTO image_metas (resource_id, width, height, file_format, file_size_bytes) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT(resource_id) DO UPDATE SET \
               width = EXCLUDED.width, \
               height = EXCLUDED.height, \
               file_format = EXCLUDED.file_format, \
               file_size_bytes = EXCLUDED.file_size_bytes",
        )
        .bind(resource_id.to_string())
        .bind(width)
        .bind(height)
        .bind(&input.file_format)
        .bind(file_size_bytes)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(ImageMeta {
            resource_id,
            width: input.width,
            height: input.height,
            file_format: input.file_format,
            file_size_bytes: input.file_size_bytes,
        })
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgImageMetaRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl ImageMetaRepository for PgImageMetaRepository {
    async fn get(&self, _resource_id: Uuid) -> Result<ImageMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn upsert(&self, _resource_id: Uuid, _input: NewImageMeta) -> Result<ImageMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
