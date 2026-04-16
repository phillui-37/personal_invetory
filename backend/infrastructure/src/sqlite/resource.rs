use crate::sqlite::{
    decode_resource_type, encode_resource_type, map_sqlite_error, parse_timestamp,
    SharedSqliteConnection,
};
use async_trait::async_trait;
use chrono::Utc;
use domain::{
    DomainError, NewResource, Resource, ResourceRepository, UpdateResource,
};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

pub struct SqliteResourceRepository {
    conn: SharedSqliteConnection,
}

impl SqliteResourceRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }

    fn title_exists(
        &self,
        title: &str,
        exclude_id: Option<Uuid>,
    ) -> Result<bool, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        let exclude_id = exclude_id.map(|id| id.to_string());
        let mut stmt = conn
            .prepare(
                "SELECT 1
                 FROM resources
                 WHERE lower(title) = lower(?1)
                   AND (?2 IS NULL OR id != ?2)
                 LIMIT 1",
            )
            .map_err(map_sqlite_error)?;

        let found = stmt
            .query_row(params![title, exclude_id], |row| row.get::<_, i64>(0))
            .optional()
            .map_err(map_sqlite_error)?
            .is_some();
        Ok(found)
    }

    fn row_to_resource(row: &rusqlite::Row<'_>) -> rusqlite::Result<Resource> {
        let id_raw: String = row.get(0)?;
        let title: String = row.get(1)?;
        let notes: Option<String> = row.get(2)?;
        let resource_type_raw: String = row.get(3)?;
        let created_at_raw: String = row.get(4)?;
        let updated_at_raw: String = row.get(5)?;

        let id = Uuid::parse_str(&id_raw).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    error.to_string(),
                )),
            )
        })?;

        let resource_type = decode_resource_type(resource_type_raw)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{error:?}")))))?;

        let created_at = parse_timestamp(created_at_raw)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{error:?}")))))?;
        let updated_at = parse_timestamp(updated_at_raw)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{error:?}")))))?;

        Ok(Resource {
            id,
            title,
            notes,
            resource_type,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl ResourceRepository for SqliteResourceRepository {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, notes, resource_type, created_at, updated_at
                 FROM resources
                 ORDER BY title COLLATE NOCASE ASC, id ASC",
            )
            .map_err(map_sqlite_error)?;

        let rows = stmt
            .query_map([], Self::row_to_resource)
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(rows)
    }

    async fn search(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        let normalized = query.trim();
        if normalized.is_empty() {
            return Err(DomainError::ValidationError(
                "query cannot be empty".to_string(),
            ));
        }

        let pattern = format!("%{normalized}%");
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, notes, resource_type, created_at, updated_at
                 FROM resources
                 WHERE lower(title) LIKE lower(?1)
                 ORDER BY title COLLATE NOCASE ASC, id ASC",
            )
            .map_err(map_sqlite_error)?;

        let rows = stmt
            .query_map(params![pattern], Self::row_to_resource)
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(rows)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;
        conn.query_row(
            "SELECT id, title, notes, resource_type, created_at, updated_at
             FROM resources
             WHERE id = ?1",
            params![id.to_string()],
            Self::row_to_resource,
        )
        .map_err(map_sqlite_error)
    }

    async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
        if self.title_exists(&input.title, None)? {
            return Err(DomainError::Conflict(
                "resource title already exists".to_string(),
            ));
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let now_raw = now.to_rfc3339();
        let resource_type = input.resource_type;
        let title = input.title;
        let notes = input.notes;

        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO resources (id, title, notes, resource_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id.to_string(),
                title.as_str(),
                notes.as_deref(),
                encode_resource_type(&resource_type),
                now_raw,
                now_raw
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(Resource {
            id,
            title,
            notes,
            resource_type,
            created_at: now,
            updated_at: now,
        })
    }

    async fn update(&self, id: Uuid, input: UpdateResource) -> Result<Resource, DomainError> {
        let existing = self.get_by_id(id).await?;

        let next_title = input.title.unwrap_or(existing.title);
        if self.title_exists(&next_title, Some(id))? {
            return Err(DomainError::Conflict(
                "resource title already exists".to_string(),
            ));
        }

        let next_notes = input.notes.or(existing.notes);
        let updated_at = Utc::now();
        let updated_at_raw = updated_at.to_rfc3339();

        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        let affected = conn
            .execute(
                "UPDATE resources
                 SET title = ?1,
                     notes = ?2,
                     updated_at = ?3
                 WHERE id = ?4",
                params![
                    next_title.as_str(),
                    next_notes.as_deref(),
                    updated_at_raw,
                    id.to_string()
                ],
            )
            .map_err(map_sqlite_error)?;

        if affected == 0 {
            return Err(DomainError::NotFound(format!("resource {id} not found")));
        }

        Ok(Resource {
            id,
            title: next_title,
            notes: next_notes,
            resource_type: existing.resource_type,
            created_at: existing.created_at,
            updated_at,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        let affected = conn
            .execute("DELETE FROM resources WHERE id = ?1", params![id.to_string()])
            .map_err(map_sqlite_error)?;

        if affected == 0 {
            return Err(DomainError::NotFound(format!("resource {id} not found")));
        }

        Ok(())
    }
}
