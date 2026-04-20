use async_trait::async_trait;
use domain::{ChapterCheck, ChapterCheckRepository, DomainError};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    crate::postgres::error::pg_err,
    chrono::Utc,
    sqlx::{PgPool, Row},
};

#[cfg(feature = "postgres")]
pub struct PgChapterCheckRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgChapterCheckRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl ChapterCheckRepository for PgChapterCheckRepository {
    async fn create(
        &self,
        resource_id: Uuid,
        has_new_chapter: bool,
        latest_chapter: Option<String>,
        error_message: Option<String>,
    ) -> Result<ChapterCheck, DomainError> {
        let id = Uuid::new_v4();
        let checked_at = Utc::now();
        let checked_at_str = checked_at.to_rfc3339();
        sqlx::query(
            "INSERT INTO chapter_checks (id, resource_id, has_new_chapter, latest_chapter, checked_at, error_message) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id.to_string())
        .bind(resource_id.to_string())
        .bind(if has_new_chapter { 1i64 } else { 0i64 })
        .bind(&latest_chapter)
        .bind(&checked_at_str)
        .bind(&error_message)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(ChapterCheck {
            id,
            resource_id,
            has_new_chapter,
            latest_chapter,
            checked_at,
            error_message,
        })
    }

    async fn list(&self, resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, has_new_chapter, latest_chapter, checked_at, error_message \
             FROM chapter_checks WHERE resource_id = $1 ORDER BY checked_at DESC",
        )
        .bind(resource_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;

        rows.iter()
            .map(|row| {
                let id_s: String = row.try_get("id").map_err(pg_err)?;
                let id = Uuid::parse_str(&id_s)
                    .map_err(|e| DomainError::InternalError(e.to_string()))?;
                let hnc: i64 = row.try_get("has_new_chapter").map_err(pg_err)?;
                let ts: String = row.try_get("checked_at").map_err(pg_err)?;
                let checked_at = chrono::DateTime::parse_from_rfc3339(&ts)
                    .map(|d| d.with_timezone(&Utc))
                    .map_err(|e| DomainError::InternalError(format!("invalid timestamp: {e}")))?;
                Ok(ChapterCheck {
                    id,
                    resource_id,
                    has_new_chapter: hnc != 0,
                    latest_chapter: row.try_get("latest_chapter").map_err(pg_err)?,
                    checked_at,
                    error_message: row.try_get("error_message").map_err(pg_err)?,
                })
            })
            .collect()
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgChapterCheckRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl ChapterCheckRepository for PgChapterCheckRepository {
    async fn create(
        &self,
        _resource_id: Uuid,
        _has_new_chapter: bool,
        _latest_chapter: Option<String>,
        _error_message: Option<String>,
    ) -> Result<ChapterCheck, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }

    async fn list(&self, _resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
