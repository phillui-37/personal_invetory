use crate::sqlite::{map_sqlite_error, SharedSqliteConnection};
use async_trait::async_trait;
use domain::{DomainError, ImageMeta, ImageMetaRepository, NewImageMeta};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

pub struct SqliteImageMetaRepository {
    conn: SharedSqliteConnection,
}

impl SqliteImageMetaRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl ImageMetaRepository for SqliteImageMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<ImageMeta, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.query_row(
            "SELECT width, height, file_format, file_size_bytes
             FROM image_metas
             WHERE resource_id = ?1",
            params![resource_id.to_string()],
            |row| {
                Ok(ImageMeta {
                    resource_id,
                    width: row
                        .get::<_, Option<i64>>(0)?
                        .map(|v| u32::try_from(v).unwrap_or(0)),
                    height: row
                        .get::<_, Option<i64>>(1)?
                        .map(|v| u32::try_from(v).unwrap_or(0)),
                    file_format: row.get(2)?,
                    file_size_bytes: row
                        .get::<_, Option<i64>>(3)?
                        .map(|v| u64::try_from(v).unwrap_or(0)),
                })
            },
        )
        .optional()
        .map_err(map_sqlite_error)?
        .ok_or_else(|| DomainError::NotFound(format!("image meta {resource_id} not found")))
    }

    async fn upsert(&self, resource_id: Uuid, input: NewImageMeta) -> Result<ImageMeta, DomainError> {
        let width = input.width.map(i64::from);
        let height = input.height.map(i64::from);
        let file_format = input.file_format;
        let file_size_bytes = input
            .file_size_bytes
            .map(|v| i64::try_from(v).unwrap_or(i64::MAX));
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO image_metas (resource_id, width, height, file_format, file_size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(resource_id) DO UPDATE SET
                 width = excluded.width,
                 height = excluded.height,
                 file_format = excluded.file_format,
                 file_size_bytes = excluded.file_size_bytes",
            params![
                resource_id.to_string(),
                width,
                height,
                file_format.as_deref(),
                file_size_bytes
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(ImageMeta {
            resource_id,
            width: input.width,
            height: input.height,
            file_format,
            file_size_bytes: input.file_size_bytes,
        })
    }
}
