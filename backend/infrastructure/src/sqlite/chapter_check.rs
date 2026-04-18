use crate::sqlite::{map_sqlite_error, parse_timestamp, SharedSqliteConnection};
use async_trait::async_trait;
use chrono::Utc;
use domain::{ChapterCheck, ChapterCheckRepository, DomainError};
use rusqlite::params;
use uuid::Uuid;

pub struct SqliteChapterCheckRepository {
    conn: SharedSqliteConnection,
}

impl SqliteChapterCheckRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl ChapterCheckRepository for SqliteChapterCheckRepository {
    async fn create(
        &self,
        resource_id: Uuid,
        has_new_chapter: bool,
        latest_chapter: Option<String>,
        error_message: Option<String>,
    ) -> Result<ChapterCheck, DomainError> {
        let id = Uuid::new_v4();
        let checked_at = Utc::now();
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO chapter_checks (id, resource_id, has_new_chapter, latest_chapter, checked_at, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id.to_string(),
                resource_id.to_string(),
                has_new_chapter as i32,
                latest_chapter.as_deref(),
                checked_at.to_rfc3339(),
                error_message.as_deref()
            ],
        )
        .map_err(map_sqlite_error)?;

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
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, has_new_chapter, latest_chapter, checked_at, error_message
                 FROM chapter_checks
                 WHERE resource_id = ?1
                 ORDER BY checked_at DESC",
            )
            .map_err(map_sqlite_error)?;

        let checks = stmt
            .query_map(params![resource_id.to_string()], |row| {
                let id_str: String = row.get(0)?;
                let has_new_chapter: i32 = row.get(1)?;
                let latest_chapter: Option<String> = row.get(2)?;
                let checked_at_str: String = row.get(3)?;
                let error_message: Option<String> = row.get(4)?;
                Ok((id_str, has_new_chapter, latest_chapter, checked_at_str, error_message))
            })
            .map_err(map_sqlite_error)?
            .map(|row| {
                let (id_str, has_new, latest_chapter, checked_at_str, error_message) =
                    row.map_err(map_sqlite_error)?;
                let id = Uuid::parse_str(&id_str)
                    .map_err(|e| DomainError::InternalError(format!("invalid uuid: {e}")))?;
                let checked_at = parse_timestamp(checked_at_str)?;
                Ok(ChapterCheck {
                    id,
                    resource_id,
                    has_new_chapter: has_new != 0,
                    latest_chapter,
                    checked_at,
                    error_message,
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;

        Ok(checks)
    }
}
