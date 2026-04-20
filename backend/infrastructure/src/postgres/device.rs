use async_trait::async_trait;
use domain::device::{Device, DeviceRepository};
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
pub struct PgDeviceRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgDeviceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
fn row_to_device(r: &sqlx::postgres::PgRow) -> Result<Device, DomainError> {
    use sqlx::Row as _;
    let linked_at_raw: String = r.try_get("linked_at").map_err(pg_err)?;
    let delinked_at_raw: Option<String> = r.try_get("delinked_at").map_err(pg_err)?;
    let location_count: i64 = r.try_get("location_count").map_err(pg_err)?;
    Ok(Device {
        id: r.try_get("id").map_err(pg_err)?,
        device_id: r.try_get("device_id").map_err(pg_err)?,
        device_name: r.try_get("device_name").map_err(pg_err)?,
        linked_at: parse_ts(&linked_at_raw)?,
        delinked_at: delinked_at_raw.map(|s| parse_ts(&s)).transpose()?,
        location_count: u64::try_from(location_count).unwrap_or(0),
    })
}

#[cfg(feature = "postgres")]
#[async_trait]
impl DeviceRepository for PgDeviceRepository {
    async fn all_with_counts(&self) -> Result<Vec<Device>, DomainError> {
        let rows = sqlx::query(
            "SELECT d.id, d.device_id, d.device_name, d.linked_at, d.delinked_at,
                    COALESCE(v.location_count, 0) AS location_count
             FROM devices d
             LEFT JOIN v_device_location_counts v ON v.device_id = d.device_id
             WHERE d.id = (
                 SELECT d2.id FROM devices d2
                 WHERE d2.device_id = d.device_id
                 ORDER BY d2.linked_at DESC LIMIT 1
             )
             ORDER BY d.linked_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;
        rows.iter().map(row_to_device).collect()
    }

    async fn active_by_device_id(&self, device_id: &str) -> Result<Option<Device>, DomainError> {
        let row = sqlx::query(
            "SELECT d.id, d.device_id, d.device_name, d.linked_at, d.delinked_at,
                    COALESCE(v.location_count, 0) AS location_count
             FROM devices d
             LEFT JOIN v_device_location_counts v ON v.device_id = d.device_id
             WHERE d.device_id = $1 AND d.delinked_at IS NULL
             ORDER BY d.linked_at DESC LIMIT 1",
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?;
        row.map(|r| row_to_device(&r)).transpose()
    }

    async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<Device, DomainError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        // Delink existing active + insert new in a transaction
        let mut tx = self.pool.begin().await.map_err(pg_err)?;
        sqlx::query(
            "UPDATE devices SET delinked_at = $1 WHERE device_id = $2 AND delinked_at IS NULL",
        )
        .bind(&now_str)
        .bind(device_id)
        .execute(&mut *tx)
        .await
        .map_err(pg_err)?;
        sqlx::query(
            "INSERT INTO devices (id, device_id, owner_id, linked_at, device_name)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&id)
        .bind(device_id)
        .bind(device_id)
        .bind(&now_str)
        .bind(device_name)
        .execute(&mut *tx)
        .await
        .map_err(pg_err)?;
        tx.commit().await.map_err(pg_err)?;
        Ok(Device {
            id,
            device_id: device_id.to_string(),
            device_name: device_name.map(str::to_string),
            linked_at: now,
            delinked_at: None,
            location_count: 0,
        })
    }

    async fn delink(&self, device_id: &str, at: DateTime<Utc>) -> Result<(), DomainError> {
        let count_row = sqlx::query("SELECT COUNT(*) AS cnt FROM devices WHERE device_id = $1")
            .bind(device_id)
            .fetch_one(&self.pool)
            .await
            .map_err(pg_err)?;
        let total: i64 = count_row.try_get("cnt").map_err(pg_err)?;
        if total == 0 {
            return Err(DomainError::NotFound(format!("device '{device_id}' not found")));
        }
        let active_row = sqlx::query(
            "SELECT COUNT(*) AS cnt FROM devices WHERE device_id = $1 AND delinked_at IS NULL",
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await
        .map_err(pg_err)?;
        let active: i64 = active_row.try_get("cnt").map_err(pg_err)?;
        if active == 0 {
            return Err(DomainError::Conflict(format!("device '{device_id}' is already delinked")));
        }
        let at_str = at.to_rfc3339();
        sqlx::query(
            "UPDATE devices SET delinked_at = $1 WHERE device_id = $2 AND delinked_at IS NULL",
        )
        .bind(&at_str)
        .bind(device_id)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(())
    }
}

#[cfg(not(feature = "postgres"))]
pub fn _placeholder() {}

