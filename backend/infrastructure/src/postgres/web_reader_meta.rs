use async_trait::async_trait;
use domain::{DomainError, NewWebReaderMeta, WebReaderMeta, WebReaderMetaRepository};
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
        .map_err(|e| DomainError::InternalError(format!("invalid timestamp '{s}': {e}")))
}

#[cfg(feature = "postgres")]
pub struct PgWebReaderMetaRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgWebReaderMetaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl WebReaderMetaRepository for PgWebReaderMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
        let row = sqlx::query(
            "SELECT resource_id, url, site_name, last_checked_chapter, \
                    check_interval_secs, last_checked_at, progress_css_selector \
             FROM web_reader_metas WHERE resource_id = $1",
        )
        .bind(resource_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| {
            DomainError::NotFound(format!(
                "web reader meta for resource {resource_id} not found"
            ))
        })?;

        let rid: String = row.try_get("resource_id").map_err(pg_err)?;
        let interval: Option<i64> = row.try_get("check_interval_secs").map_err(pg_err)?;
        let checked_at_s: Option<String> = row.try_get("last_checked_at").map_err(pg_err)?;
        let last_checked_at = checked_at_s
            .as_deref()
            .map(parse_ts)
            .transpose()?;

        Ok(WebReaderMeta {
            resource_id: Uuid::parse_str(&rid)
                .map_err(|e| DomainError::InternalError(e.to_string()))?,
            url: row.try_get("url").map_err(pg_err)?,
            site_name: row.try_get("site_name").map_err(pg_err)?,
            last_checked_chapter: row.try_get("last_checked_chapter").map_err(pg_err)?,
            check_interval_secs: interval.map(|v| v as u64),
            last_checked_at,
            progress_css_selector: row.try_get("progress_css_selector").map_err(pg_err)?,
        })
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewWebReaderMeta,
    ) -> Result<WebReaderMeta, DomainError> {
        let interval: Option<i64> = input.check_interval_secs.map(|v| v as i64);
        let checked_at_s: Option<String> = input.last_checked_at.map(|dt| dt.to_rfc3339());

        sqlx::query(
            "INSERT INTO web_reader_metas \
               (resource_id, url, site_name, last_checked_chapter, \
                check_interval_secs, last_checked_at, progress_css_selector) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT(resource_id) DO UPDATE SET \
               url = EXCLUDED.url, \
               site_name = EXCLUDED.site_name, \
               last_checked_chapter = EXCLUDED.last_checked_chapter, \
               check_interval_secs = EXCLUDED.check_interval_secs, \
               last_checked_at = EXCLUDED.last_checked_at, \
               progress_css_selector = EXCLUDED.progress_css_selector",
        )
        .bind(resource_id.to_string())
        .bind(&input.url)
        .bind(&input.site_name)
        .bind(&input.last_checked_chapter)
        .bind(interval)
        .bind(&checked_at_s)
        .bind(&input.progress_css_selector)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(WebReaderMeta {
            resource_id,
            url: input.url,
            site_name: input.site_name,
            last_checked_chapter: input.last_checked_chapter,
            check_interval_secs: input.check_interval_secs,
            last_checked_at: input.last_checked_at,
            progress_css_selector: input.progress_css_selector,
        })
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgWebReaderMetaRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl WebReaderMetaRepository for PgWebReaderMetaRepository {
    async fn get(&self, _resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn upsert(
        &self,
        _resource_id: Uuid,
        _input: NewWebReaderMeta,
    ) -> Result<WebReaderMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
