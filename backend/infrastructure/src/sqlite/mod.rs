pub mod chapter_check;
pub mod dedup;
pub mod device;
pub mod ebook_meta;
pub mod game_meta;
pub mod image_meta;
pub mod location;
pub mod notification;
pub mod resource;
pub mod sync_job;
pub mod vault;
pub mod video_meta;
pub mod web_reader_meta;

use chrono::{DateTime, Utc};
use domain::{DomainError, ResourceType, StorageType};
use rusqlite::{Connection, Error as SqlError, ErrorCode, OpenFlags};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub type SharedSqliteConnection = Arc<Mutex<Connection>>;

const SQLITE_MIGRATIONS: [&str; 18] = [
    include_str!("../../migrations/0001_create_resources.sql"),
    include_str!("../../migrations/0002_create_ebook_metas.sql"),
    include_str!("../../migrations/0003_create_web_reader_metas.sql"),
    include_str!("../../migrations/0004_create_resource_locations.sql"),
    include_str!("../../migrations/0005_create_devices.sql"),
    include_str!("../../migrations/0006_alter_web_reader_metas_add_check_fields.sql"),
    include_str!("../../migrations/0007_create_chapter_checks.sql"),
    include_str!("../../migrations/0008_create_site_configs.sql"),
    include_str!("../../migrations/0009_create_notifications.sql"),
    include_str!("../../migrations/0010_create_image_metas.sql"),
    include_str!("../../migrations/0011_create_video_metas.sql"),
    include_str!("../../migrations/0012_create_game_metas.sql"),
    include_str!("../../migrations/0013_create_vault_config.sql"),
    include_str!("../../migrations/0014_create_credentials.sql"),
    include_str!("../../migrations/0015_create_sync_jobs.sql"),
    include_str!("../../migrations/0016_create_dedup_warnings.sql"),
    include_str!("../../migrations/0017_alter_devices_add_name.sql"),
    include_str!("../../migrations/0018_create_device_location_view.sql"),
];

pub fn open_sqlite_connection(database_url: &str) -> Result<Connection, DomainError> {
    let path = database_url.strip_prefix("sqlite://").ok_or_else(|| {
        DomainError::ValidationError("sqlite URL must start with sqlite://".to_string())
    })?;

    if path.is_empty() {
        return Err(DomainError::ValidationError(
            "sqlite URL path cannot be empty".to_string(),
        ));
    }

    let conn = if path == ":memory:" {
        Connection::open_in_memory().map_err(map_sqlite_error)?
    } else if path.starts_with("file:") {
        Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(map_sqlite_error)?
    } else {
        Connection::open(path).map_err(map_sqlite_error)?
    };

    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(map_sqlite_error)?;
    apply_sqlite_migrations(&conn)?;

    Ok(conn)
}

pub fn apply_sqlite_migrations(conn: &Connection) -> Result<(), DomainError> {
    for migration_sql in SQLITE_MIGRATIONS {
        conn.execute_batch(migration_sql)
            .map_err(map_sqlite_error)?;
    }
    Ok(())
}

pub fn map_sqlite_error(error: SqlError) -> DomainError {
    match error {
        SqlError::QueryReturnedNoRows => DomainError::NotFound("record not found".to_string()),
        SqlError::SqliteFailure(err, message) => {
            if err.code == ErrorCode::ConstraintViolation {
                let reason = message.unwrap_or_else(|| "constraint violation".to_string());
                if reason.to_ascii_uppercase().contains("FOREIGN KEY") {
                    DomainError::NotFound(reason)
                } else {
                    DomainError::Conflict(reason)
                }
            } else {
                DomainError::InternalError(message.unwrap_or_else(|| err.to_string()))
            }
        }
        other => DomainError::InternalError(other.to_string()),
    }
}

pub fn encode_resource_type(resource_type: &ResourceType) -> &'static str {
    match resource_type {
        ResourceType::Ebook => "ebook",
        ResourceType::WebReader => "web_reader",
        ResourceType::Image => "image",
        ResourceType::Video => "video",
        ResourceType::Game => "game",
    }
}

pub fn decode_resource_type(raw: String) -> Result<ResourceType, DomainError> {
    match raw.as_str() {
        "ebook" | "Ebook" => Ok(ResourceType::Ebook),
        "web_reader" | "WebReader" => Ok(ResourceType::WebReader),
        "image" | "Image" => Ok(ResourceType::Image),
        "video" | "Video" => Ok(ResourceType::Video),
        "game" | "Game" => Ok(ResourceType::Game),
        _ => Err(DomainError::InternalError(format!(
            "unknown resource_type value: {raw}"
        ))),
    }
}

pub fn encode_storage_type(storage_type: &StorageType) -> &'static str {
    match storage_type {
        StorageType::LocalFs => "LocalFs",
        StorageType::Nas => "Nas",
        StorageType::Platform => "Platform",
        StorageType::Portable => "Portable",
    }
}

pub fn decode_storage_type(raw: String) -> Result<StorageType, DomainError> {
    match raw.as_str() {
        "LocalFs" | "local_fs" | "localfs" => Ok(StorageType::LocalFs),
        "Nas" | "nas" => Ok(StorageType::Nas),
        "Platform" | "platform" => Ok(StorageType::Platform),
        "Portable" | "portable" => Ok(StorageType::Portable),
        _ => Err(DomainError::InternalError(format!(
            "unknown storage_type value: {raw}"
        ))),
    }
}

pub fn parse_timestamp(raw: String) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(&raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| DomainError::InternalError(format!("invalid timestamp '{raw}': {error}")))
}

pub fn parse_uuid_for_row(raw: &str) -> Result<Uuid, rusqlite::Error> {
    Uuid::parse_str(raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })
}

pub fn parse_timestamp_for_row(raw: String) -> Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(&raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })
}
