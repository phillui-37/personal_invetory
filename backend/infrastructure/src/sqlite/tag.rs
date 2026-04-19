use async_trait::async_trait;
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::sqlite::{map_sqlite_error, parse_timestamp_for_row, SharedSqliteConnection};
use domain::tag::{ResourceTagRepository, Tag, TagRepository};
use domain::DomainError;

pub struct SqliteTagRepository {
    conn: SharedSqliteConnection,
}

impl SqliteTagRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }

    fn row_to_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let created_at_raw: String = row.get(2)?;
        let created_at = parse_timestamp_for_row(created_at_raw)?;
        Ok(Tag { id, name, created_at })
    }
}

#[async_trait]
impl TagRepository for SqliteTagRepository {
    async fn list(&self) -> Result<Vec<Tag>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, created_at FROM tags ORDER BY name COLLATE NOCASE ASC, id ASC",
            )
            .map_err(map_sqlite_error)?;
        let rows = stmt
            .query_map([], Self::row_to_tag)
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(rows)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Tag>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        conn.query_row(
            "SELECT id, name, created_at FROM tags WHERE id = ?1",
            params![id],
            Self::row_to_tag,
        )
        .optional()
        .map_err(map_sqlite_error)
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Tag>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        conn.query_row(
            "SELECT id, name, created_at FROM tags WHERE name = ?1",
            params![name],
            Self::row_to_tag,
        )
        .optional()
        .map_err(map_sqlite_error)
    }

    async fn create(&self, name: &str) -> Result<Tag, DomainError> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| DomainError::InternalError(e.to_string()))?;
            conn.execute(
                "INSERT INTO tags (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, name, created_at],
            )
            .map_err(map_sqlite_error)?;
        }
        self.get_by_id(&id)
            .await?
            .ok_or_else(|| DomainError::InternalError(format!("tag {id} not found after insert")))
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let affected = conn
            .execute("DELETE FROM tags WHERE id = ?1", params![id])
            .map_err(map_sqlite_error)?;
        if affected == 0 {
            return Err(DomainError::NotFound(format!("tag {id} not found")));
        }
        Ok(())
    }
}

#[async_trait]
impl ResourceTagRepository for SqliteTagRepository {
    async fn tags_for_resource(&self, resource_id: &str) -> Result<Vec<Tag>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.name, t.created_at
                 FROM tags t
                 JOIN resource_tags rt ON rt.tag_id = t.id
                 WHERE rt.resource_id = ?1
                 ORDER BY t.name COLLATE NOCASE ASC, t.id ASC",
            )
            .map_err(map_sqlite_error)?;
        let rows = stmt
            .query_map(params![resource_id], Self::row_to_tag)
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(rows)
    }

    async fn attach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO resource_tags (resource_id, tag_id) VALUES (?1, ?2)",
            params![resource_id, tag_id],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn detach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let affected = conn
            .execute(
                "DELETE FROM resource_tags WHERE resource_id = ?1 AND tag_id = ?2",
                params![resource_id, tag_id],
            )
            .map_err(map_sqlite_error)?;
        if affected == 0 {
            return Err(DomainError::NotFound(format!(
                "resource_tag association ({resource_id}, {tag_id}) not found"
            )));
        }
        Ok(())
    }

    async fn resource_ids_with_tag_id(&self, tag_id: &str) -> Result<Vec<String>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT resource_id FROM resource_tags WHERE tag_id = ?1 ORDER BY resource_id ASC",
            )
            .map_err(map_sqlite_error)?;
        let rows = stmt
            .query_map(params![tag_id], |row| row.get::<_, String>(0))
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(rows)
    }
}
