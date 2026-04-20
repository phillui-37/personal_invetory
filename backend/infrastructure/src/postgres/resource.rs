use async_trait::async_trait;
use domain::{DomainError, NewResource, Resource, ResourceRepository, UpdateResource};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    crate::postgres::error::pg_err,
    chrono::{DateTime, Utc},
    sqlx::{PgPool, Row},
};

#[cfg(feature = "postgres")]
fn encode_resource_type(rt: &ResourceType) -> &'static str {
    match rt {
        ResourceType::Ebook => "Ebook",
        ResourceType::WebReader => "WebReader",
        ResourceType::Image => "Image",
        ResourceType::Video => "Video",
        ResourceType::Game => "Game",
    }
}

#[cfg(feature = "postgres")]
fn decode_resource_type(s: &str) -> Result<ResourceType, DomainError> {
    match s {
        "Ebook" | "ebook" => Ok(ResourceType::Ebook),
        "WebReader" | "web_reader" => Ok(ResourceType::WebReader),
        "Image" | "image" => Ok(ResourceType::Image),
        "Video" | "video" => Ok(ResourceType::Video),
        "Game" | "game" => Ok(ResourceType::Game),
        other => Err(DomainError::InternalError(format!(
            "unknown resource_type: {other}"
        ))),
    }
}

#[cfg(feature = "postgres")]
fn parse_ts(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| DomainError::InternalError(format!("invalid timestamp '{s}': {e}")))
}

#[cfg(feature = "postgres")]
pub struct PgResourceRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgResourceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn title_exists(&self, title: &str, exclude_id: Option<Uuid>) -> Result<bool, DomainError> {
        let exclude = exclude_id.map(|id| id.to_string());
        let row = sqlx::query(
            "SELECT 1 FROM resources WHERE LOWER(title) = LOWER($1) AND ($2::TEXT IS NULL OR id != $2) LIMIT 1",
        )
        .bind(title)
        .bind(exclude)
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(row.is_some())
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl ResourceRepository for PgResourceRepository {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, title, notes, resource_type, created_at, updated_at \
             FROM resources ORDER BY LOWER(title) ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;

        rows.iter()
            .map(|row| {
                let id: String = row.try_get("id").map_err(pg_err)?;
                let rt: String = row.try_get("resource_type").map_err(pg_err)?;
                let ca: String = row.try_get("created_at").map_err(pg_err)?;
                let ua: String = row.try_get("updated_at").map_err(pg_err)?;
                Ok(Resource {
                    id: Uuid::parse_str(&id)
                        .map_err(|e| DomainError::InternalError(e.to_string()))?,
                    title: row.try_get("title").map_err(pg_err)?,
                    notes: row.try_get("notes").map_err(pg_err)?,
                    resource_type: decode_resource_type(&rt)?,
                    created_at: parse_ts(&ca)?,
                    updated_at: parse_ts(&ua)?,
                })
            })
            .collect()
    }

    async fn search(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        let normalized = query.trim();
        if normalized.is_empty() {
            return Err(DomainError::ValidationError(
                "query cannot be empty".to_string(),
            ));
        }
        let pattern = format!("%{normalized}%");
        let rows = sqlx::query(
            "SELECT id, title, notes, resource_type, created_at, updated_at \
             FROM resources WHERE LOWER(title) LIKE LOWER($1) ORDER BY LOWER(title) ASC, id ASC",
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;

        rows.iter()
            .map(|row| {
                let id: String = row.try_get("id").map_err(pg_err)?;
                let rt: String = row.try_get("resource_type").map_err(pg_err)?;
                let ca: String = row.try_get("created_at").map_err(pg_err)?;
                let ua: String = row.try_get("updated_at").map_err(pg_err)?;
                Ok(Resource {
                    id: Uuid::parse_str(&id)
                        .map_err(|e| DomainError::InternalError(e.to_string()))?,
                    title: row.try_get("title").map_err(pg_err)?,
                    notes: row.try_get("notes").map_err(pg_err)?,
                    resource_type: decode_resource_type(&rt)?,
                    created_at: parse_ts(&ca)?,
                    updated_at: parse_ts(&ua)?,
                })
            })
            .collect()
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
        let row = sqlx::query(
            "SELECT id, title, notes, resource_type, created_at, updated_at \
             FROM resources WHERE id = $1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| DomainError::NotFound(format!("resource {id} not found")))?;

        let id_s: String = row.try_get("id").map_err(pg_err)?;
        let rt: String = row.try_get("resource_type").map_err(pg_err)?;
        let ca: String = row.try_get("created_at").map_err(pg_err)?;
        let ua: String = row.try_get("updated_at").map_err(pg_err)?;
        Ok(Resource {
            id: Uuid::parse_str(&id_s).map_err(|e| DomainError::InternalError(e.to_string()))?,
            title: row.try_get("title").map_err(pg_err)?,
            notes: row.try_get("notes").map_err(pg_err)?,
            resource_type: decode_resource_type(&rt)?,
            created_at: parse_ts(&ca)?,
            updated_at: parse_ts(&ua)?,
        })
    }

    async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
        if self.title_exists(&input.title, None).await? {
            return Err(DomainError::Conflict(
                "resource title already exists".to_string(),
            ));
        }
        let id = Uuid::new_v4();
        let now = Utc::now();
        let now_s = now.to_rfc3339();
        sqlx::query(
            "INSERT INTO resources (id, title, notes, resource_type, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id.to_string())
        .bind(&input.title)
        .bind(&input.notes)
        .bind(encode_resource_type(&input.resource_type))
        .bind(&now_s)
        .bind(&now_s)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(Resource {
            id,
            title: input.title,
            notes: input.notes,
            resource_type: input.resource_type,
            created_at: now,
            updated_at: now,
        })
    }

    async fn update(&self, id: Uuid, input: UpdateResource) -> Result<Resource, DomainError> {
        let existing = self.get_by_id(id).await?;
        let next_title = input.title.unwrap_or(existing.title.clone());
        if self.title_exists(&next_title, Some(id)).await? {
            return Err(DomainError::Conflict(
                "resource title already exists".to_string(),
            ));
        }
        let next_notes = input.notes.or(existing.notes.clone());
        let updated_at = Utc::now();
        let updated_at_s = updated_at.to_rfc3339();

        let affected = sqlx::query(
            "UPDATE resources SET title = $1, notes = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(&next_title)
        .bind(&next_notes)
        .bind(&updated_at_s)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(pg_err)?
        .rows_affected();

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
        let affected = sqlx::query("DELETE FROM resources WHERE id = $1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(pg_err)?
            .rows_affected();

        if affected == 0 {
            return Err(DomainError::NotFound(format!("resource {id} not found")));
        }
        Ok(())
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgResourceRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl ResourceRepository for PgResourceRepository {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn search(&self, _query: &str) -> Result<Vec<Resource>, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn get_by_id(&self, _id: Uuid) -> Result<Resource, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn create(&self, _input: NewResource) -> Result<Resource, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn update(&self, _id: Uuid, _input: UpdateResource) -> Result<Resource, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
