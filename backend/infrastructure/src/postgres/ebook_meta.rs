use async_trait::async_trait;
use domain::{DomainError, EbookMeta, EbookMetaRepository, NewEbookMeta};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {crate::postgres::error::pg_err, sqlx::{PgPool, Row}};

#[cfg(feature = "postgres")]
pub struct PgEbookMetaRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgEbookMetaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl EbookMetaRepository for PgEbookMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        let row = sqlx::query(
            "SELECT resource_id, author, isbn, publisher, language, file_format \
             FROM ebook_metas WHERE resource_id = $1",
        )
        .bind(resource_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| {
            DomainError::NotFound(format!("ebook meta for resource {resource_id} not found"))
        })?;

        let rid: String = row.try_get("resource_id").map_err(pg_err)?;
        Ok(EbookMeta {
            resource_id: Uuid::parse_str(&rid)
                .map_err(|e| DomainError::InternalError(e.to_string()))?,
            author: row.try_get("author").map_err(pg_err)?,
            isbn: row.try_get("isbn").map_err(pg_err)?,
            publisher: row.try_get("publisher").map_err(pg_err)?,
            language: row.try_get("language").map_err(pg_err)?,
            file_format: row.try_get("file_format").map_err(pg_err)?,
        })
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewEbookMeta,
    ) -> Result<EbookMeta, DomainError> {
        sqlx::query(
            "INSERT INTO ebook_metas (resource_id, author, isbn, publisher, language, file_format) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT(resource_id) DO UPDATE SET \
               author = EXCLUDED.author, \
               isbn = EXCLUDED.isbn, \
               publisher = EXCLUDED.publisher, \
               language = EXCLUDED.language, \
               file_format = EXCLUDED.file_format",
        )
        .bind(resource_id.to_string())
        .bind(&input.author)
        .bind(&input.isbn)
        .bind(&input.publisher)
        .bind(&input.language)
        .bind(&input.file_format)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(EbookMeta {
            resource_id,
            author: input.author,
            isbn: input.isbn,
            publisher: input.publisher,
            language: input.language,
            file_format: input.file_format,
        })
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgEbookMetaRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl EbookMetaRepository for PgEbookMetaRepository {
    async fn get(&self, _resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn upsert(
        &self,
        _resource_id: Uuid,
        _input: NewEbookMeta,
    ) -> Result<EbookMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
