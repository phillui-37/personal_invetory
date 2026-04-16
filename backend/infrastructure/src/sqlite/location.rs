use crate::sqlite::{decode_storage_type, encode_storage_type, map_sqlite_error, SharedSqliteConnection};
use async_trait::async_trait;
use domain::{DomainError, LocationRepository, NewResourceLocation, ResourceLocation};
use rusqlite::params;
use uuid::Uuid;

pub struct SqliteLocationRepository {
    conn: SharedSqliteConnection,
}

impl SqliteLocationRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl LocationRepository for SqliteLocationRepository {
    async fn list(&self, resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, device_id, path_or_url, storage_type
                 FROM resource_locations
                 WHERE resource_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(map_sqlite_error)?;

        let rows = stmt
            .query_map(params![resource_id.to_string()], |row| {
                let id_raw: String = row.get(0)?;
                let storage_raw: String = row.get(3)?;
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
                let storage_type = decode_storage_type(storage_raw).map_err(|error| {
                    rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("{error:?}"),
                    )))
                })?;

                Ok(ResourceLocation {
                    id,
                    resource_id,
                    device_id: row.get(1)?,
                    path_or_url: row.get(2)?,
                    storage_type,
                })
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;

        Ok(rows)
    }

    async fn add(
        &self,
        resource_id: Uuid,
        input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError> {
        let id = Uuid::new_v4();
        let device_id = input.device_id;
        let path_or_url = input.path_or_url;
        let storage_type = input.storage_type;
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO resource_locations (id, resource_id, device_id, path_or_url, storage_type)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id.to_string(),
                resource_id.to_string(),
                device_id.as_str(),
                path_or_url.as_str(),
                encode_storage_type(&storage_type)
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(ResourceLocation {
            id,
            resource_id,
            device_id,
            path_or_url,
            storage_type,
        })
    }

    async fn remove(&self, resource_id: Uuid, location_id: Uuid) -> Result<(), DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        let affected = conn
            .execute(
                "DELETE FROM resource_locations WHERE resource_id = ?1 AND id = ?2",
                params![resource_id.to_string(), location_id.to_string()],
            )
            .map_err(map_sqlite_error)?;

        if affected == 0 {
            return Err(DomainError::NotFound(format!(
                "location {location_id} not found for resource {resource_id}"
            )));
        }

        Ok(())
    }
}
