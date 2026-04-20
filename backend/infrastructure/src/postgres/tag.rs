use async_trait::async_trait;
use domain::tag::{ResourceTagRepository, Tag, TagRepository};
use domain::DomainError;
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    crate::postgres::error::pg_err,
    chrono::{DateTime, Utc},
    sqlx::{PgPool, Row},
};

#[cfg(feature = "postgres")]
fn parse_ts(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| DomainError::InternalError(format!("timestamp parse: {e}")))
}

#[cfg(feature = "postgres")]
fn row_to_tag(r: &sqlx::postgres::PgRow) -> Result<Tag, DomainError> {
    use sqlx::Row as _;
    let created_at_raw: String = r.try_get("created_at").map_err(pg_err)?;
    Ok(Tag {
        id: r.try_get("id").map_err(pg_err)?,
        name: r.try_get("name").map_err(pg_err)?,
        created_at: parse_ts(&created_at_raw)?,
    })
}

#[cfg(feature = "postgres")]
pub struct PgTagRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl TagRepository for PgTagRepository {
    async fn list(&self) -> Result<Vec<Tag>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, name, created_at FROM tags ORDER BY name, id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;
        rows.iter().map(row_to_tag).collect()
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Tag>, DomainError> {
        let row = sqlx::query("SELECT id, name, created_at FROM tags WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(pg_err)?;
        row.map(|r| row_to_tag(&r)).transpose()
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Tag>, DomainError> {
        let row = sqlx::query("SELECT id, name, created_at FROM tags WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(pg_err)?;
        row.map(|r| row_to_tag(&r)).transpose()
    }

    async fn create(&self, name: &str) -> Result<Tag, DomainError> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO tags (id, name, created_at) VALUES ($1, $2, $3)")
            .bind(&id)
            .bind(name)
            .bind(&created_at)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(ref db_err) = e {
                    if db_err.code().as_deref() == Some("23505") {
                        return DomainError::Conflict(format!("tag '{name}' already exists"));
                    }
                }
                pg_err(e)
            })?;
        self.get_by_id(&id)
            .await?
            .ok_or_else(|| DomainError::InternalError(format!("tag {id} not found after insert")))
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM tags WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(pg_err)?;
        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("tag {id} not found")));
        }
        Ok(())
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl ResourceTagRepository for PgTagRepository {
    async fn tags_for_resource(&self, resource_id: &str) -> Result<Vec<Tag>, DomainError> {
        let rows = sqlx::query(
            "SELECT t.id, t.name, t.created_at
             FROM tags t
             JOIN resource_tags rt ON rt.tag_id = t.id
             WHERE rt.resource_id = $1
             ORDER BY t.name, t.id ASC",
        )
        .bind(resource_id)
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;
        rows.iter().map(row_to_tag).collect()
    }

    async fn attach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO resource_tags (resource_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(resource_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(())
    }

    async fn detach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        let result = sqlx::query(
            "DELETE FROM resource_tags WHERE resource_id = $1 AND tag_id = $2",
        )
        .bind(resource_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!(
                "resource_tag association ({resource_id}, {tag_id}) not found"
            )));
        }
        Ok(())
    }

    async fn resource_ids_with_tag_id(&self, tag_id: &str) -> Result<Vec<String>, DomainError> {
        let rows = sqlx::query(
            "SELECT resource_id FROM resource_tags WHERE tag_id = $1 ORDER BY resource_id ASC",
        )
        .bind(tag_id)
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;
        rows.iter()
            .map(|r| r.try_get::<String, _>("resource_id").map_err(pg_err))
            .collect()
    }
}

#[cfg(not(feature = "postgres"))]
pub fn _placeholder() {}

