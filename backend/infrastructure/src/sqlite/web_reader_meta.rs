use crate::sqlite::{map_sqlite_error, SharedSqliteConnection};
use async_trait::async_trait;
use domain::{DomainError, NewWebReaderMeta, WebReaderMeta, WebReaderMetaRepository};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

pub struct SqliteWebReaderMetaRepository {
    conn: SharedSqliteConnection,
}

impl SqliteWebReaderMetaRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl WebReaderMetaRepository for SqliteWebReaderMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.query_row(
            "SELECT url, site_name, last_checked_chapter
             FROM web_reader_metas
             WHERE resource_id = ?1",
            params![resource_id.to_string()],
            |row| {
                Ok(WebReaderMeta {
                    resource_id,
                    url: row.get(0)?,
                    site_name: row.get(1)?,
                    last_checked_chapter: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(map_sqlite_error)?
        .ok_or_else(|| DomainError::NotFound(format!("web reader meta {resource_id} not found")))
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewWebReaderMeta,
    ) -> Result<WebReaderMeta, DomainError> {
        let url = input.url;
        let site_name = input.site_name;
        let last_checked_chapter = input.last_checked_chapter;
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO web_reader_metas (resource_id, url, site_name, last_checked_chapter)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(resource_id) DO UPDATE SET
                 url = excluded.url,
                 site_name = excluded.site_name,
                 last_checked_chapter = excluded.last_checked_chapter",
            params![
                resource_id.to_string(),
                url.as_str(),
                site_name.as_deref(),
                last_checked_chapter.as_deref()
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(WebReaderMeta {
            resource_id,
            url,
            site_name,
            last_checked_chapter,
        })
    }
}
