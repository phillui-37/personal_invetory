use crate::sqlite::{map_sqlite_error, SharedSqliteConnection};
use async_trait::async_trait;
use domain::{DomainError, EbookMeta, EbookMetaRepository, NewEbookMeta};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

pub struct SqliteEbookMetaRepository {
    conn: SharedSqliteConnection,
}

impl SqliteEbookMetaRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl EbookMetaRepository for SqliteEbookMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.query_row(
            "SELECT author, isbn, publisher, language, file_format
             FROM ebook_metas
             WHERE resource_id = ?1",
            params![resource_id.to_string()],
            |row| {
                Ok(EbookMeta {
                    resource_id,
                    author: row.get(0)?,
                    isbn: row.get(1)?,
                    publisher: row.get(2)?,
                    language: row.get(3)?,
                    file_format: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(map_sqlite_error)?
        .ok_or_else(|| DomainError::NotFound(format!("ebook meta {resource_id} not found")))
    }

    async fn upsert(&self, resource_id: Uuid, input: NewEbookMeta) -> Result<EbookMeta, DomainError> {
        let author = input.author;
        let isbn = input.isbn;
        let publisher = input.publisher;
        let language = input.language;
        let file_format = input.file_format;
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO ebook_metas (resource_id, author, isbn, publisher, language, file_format)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(resource_id) DO UPDATE SET
                 author = excluded.author,
                 isbn = excluded.isbn,
                 publisher = excluded.publisher,
                 language = excluded.language,
                 file_format = excluded.file_format",
            params![
                resource_id.to_string(),
                author.as_deref(),
                isbn.as_deref(),
                publisher.as_deref(),
                language.as_deref(),
                file_format.as_deref()
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(EbookMeta {
            resource_id,
            author,
            isbn,
            publisher,
            language,
            file_format,
        })
    }
}
