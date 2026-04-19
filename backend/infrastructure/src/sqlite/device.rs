use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use uuid::Uuid;

use domain::device::{Device, DeviceRepository};
use domain::DomainError;

use super::{map_sqlite_error, parse_timestamp_for_row, SharedSqliteConnection};

pub struct SqliteDeviceRepository {
    conn: SharedSqliteConnection,
}

impl SqliteDeviceRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

fn row_to_device(row: &rusqlite::Row<'_>) -> rusqlite::Result<Device> {
    let linked_at_str: String = row.get(3)?;
    let delinked_at_str: Option<String> = row.get(4)?;
    let location_count: i64 = row.get(5)?;
    Ok(Device {
        id: row.get(0)?,
        device_id: row.get(1)?,
        device_name: row.get(2)?,
        linked_at: parse_timestamp_for_row(linked_at_str)?,
        delinked_at: delinked_at_str.map(parse_timestamp_for_row).transpose()?,
        location_count: location_count as u64,
    })
}

#[async_trait]
impl DeviceRepository for SqliteDeviceRepository {
    async fn all_with_counts(&self) -> Result<Vec<Device>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT d.id, d.device_id, d.device_name, d.linked_at, d.delinked_at,
                        COALESCE(v.location_count, 0)
                 FROM devices d
                 LEFT JOIN v_device_location_counts v ON v.device_id = d.device_id
                 WHERE d.id = (
                     SELECT d2.id FROM devices d2
                     WHERE d2.device_id = d.device_id
                     ORDER BY d2.linked_at DESC LIMIT 1
                 )
                 ORDER BY d.linked_at DESC",
            )
            .map_err(map_sqlite_error)?;
        let rows = stmt
            .query_map([], row_to_device)
            .map_err(map_sqlite_error)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)
    }

    async fn active_by_device_id(&self, device_id: &str) -> Result<Option<Device>, DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT d.id, d.device_id, d.device_name, d.linked_at, d.delinked_at,
                    COALESCE(v.location_count, 0)
             FROM devices d
             LEFT JOIN v_device_location_counts v ON v.device_id = d.device_id
             WHERE d.device_id = ?1 AND d.delinked_at IS NULL
             ORDER BY d.linked_at DESC LIMIT 1",
            rusqlite::params![device_id],
            row_to_device,
        )
        .optional()
        .map_err(map_sqlite_error)
    }

    async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<Device, DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let id = Uuid::new_v4().to_string();

        // Delink any existing active binding
        conn.execute(
            "UPDATE devices SET delinked_at = ?1
             WHERE device_id = ?2 AND delinked_at IS NULL",
            rusqlite::params![now_str, device_id],
        )
        .map_err(map_sqlite_error)?;

        // Insert new binding (owner_id = device_id: legacy field, same value)
        conn.execute(
            "INSERT INTO devices (id, device_id, owner_id, linked_at, device_name)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, device_id, device_id, now_str, device_name],
        )
        .map_err(map_sqlite_error)?;

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
        let conn = self.conn.lock().unwrap();

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM devices WHERE device_id = ?1",
                rusqlite::params![device_id],
                |row| row.get(0),
            )
            .map_err(map_sqlite_error)?;
        if total == 0 {
            return Err(DomainError::NotFound(format!(
                "device '{device_id}' not found"
            )));
        }

        let active: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM devices
                 WHERE device_id = ?1 AND delinked_at IS NULL",
                rusqlite::params![device_id],
                |row| row.get(0),
            )
            .map_err(map_sqlite_error)?;
        if active == 0 {
            return Err(DomainError::Conflict(format!(
                "device '{device_id}' is already delinked"
            )));
        }

        let at_str = at.to_rfc3339();
        conn.execute(
            "UPDATE devices SET delinked_at = ?1
             WHERE device_id = ?2 AND delinked_at IS NULL",
            rusqlite::params![at_str, device_id],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }
}
