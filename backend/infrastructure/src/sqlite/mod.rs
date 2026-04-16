pub mod ebook_meta;
pub mod location;
pub mod resource;
pub mod web_reader_meta;

use chrono::{DateTime, Utc};
use domain::{DomainError, ResourceType, StorageType};
use rusqlite::{Connection, Error as SqlError, ErrorCode, OpenFlags};
use std::sync::{Arc, Mutex};

pub type SharedSqliteConnection = Arc<Mutex<Connection>>;

const SQLITE_MIGRATIONS: [&str; 5] = [
    include_str!("../../migrations/0001_create_resources.sql"),
    include_str!("../../migrations/0002_create_ebook_metas.sql"),
    include_str!("../../migrations/0003_create_web_reader_metas.sql"),
    include_str!("../../migrations/0004_create_resource_locations.sql"),
    include_str!("../../migrations/0005_create_devices.sql"),
];

pub fn open_sqlite_connection(database_url: &str) -> Result<Connection, DomainError> {
    let path = database_url
        .strip_prefix("sqlite://")
        .ok_or_else(|| DomainError::ValidationError("sqlite URL must start with sqlite://".to_string()))?;

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
        conn.execute_batch(migration_sql).map_err(map_sqlite_error)?;
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
    }
}

pub fn decode_resource_type(raw: String) -> Result<ResourceType, DomainError> {
    match raw.as_str() {
        "ebook" | "Ebook" => Ok(ResourceType::Ebook),
        "web_reader" | "WebReader" => Ok(ResourceType::WebReader),
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
