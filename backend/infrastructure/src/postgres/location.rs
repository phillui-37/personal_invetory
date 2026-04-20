use async_trait::async_trait;
use domain::{DomainError, LocationRepository, NewResourceLocation, ResourceLocation};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {crate::postgres::error::pg_err, sqlx::{PgPool, Row}};

#[cfg(feature = "postgres")]
fn encode_storage_type(st: &StorageType) -> &'static str {
    match st {
        StorageType::LocalFs => "LocalFs",
        StorageType::Nas => "Nas",
        StorageType::Platform => "Platform",
        StorageType::Portable => "Portable",
    }
}

#[cfg(feature = "postgres")]
fn decode_storage_type(s: &str) -> Result<StorageType, DomainError> {
    match s {
        "LocalFs" | "local_fs" | "localfs" => Ok(StorageType::LocalFs),
        "Nas" | "nas" => Ok(StorageType::Nas),
        "Platform" | "platform" => Ok(StorageType::Platform),
        "Portable" | "portable" => Ok(StorageType::Portable),
        other => Err(DomainError::InternalError(format!(
            "unknown storage_type: {other}"
        ))),
    }
}

#[cfg(feature = "postgres")]
pub struct PgLocationRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgLocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl LocationRepository for PgLocationRepository {
    async fn list(&self, resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, resource_id, device_id, path_or_url, storage_type \
             FROM resource_locations WHERE resource_id = $1 ORDER BY id ASC",
        )
        .bind(resource_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(pg_err)?;

        rows.iter()
            .map(|row| {
                let id_s: String = row.try_get("id").map_err(pg_err)?;
                let rid_s: String = row.try_get("resource_id").map_err(pg_err)?;
                let st_s: String = row.try_get("storage_type").map_err(pg_err)?;
                Ok(ResourceLocation {
                    id: Uuid::parse_str(&id_s)
                        .map_err(|e| DomainError::InternalError(e.to_string()))?,
                    resource_id: Uuid::parse_str(&rid_s)
                        .map_err(|e| DomainError::InternalError(e.to_string()))?,
                    device_id: row.try_get("device_id").map_err(pg_err)?,
                    path_or_url: row.try_get("path_or_url").map_err(pg_err)?,
                    storage_type: decode_storage_type(&st_s)?,
                })
            })
            .collect()
    }

    async fn add(
        &self,
        resource_id: Uuid,
        input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError> {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO resource_locations (id, resource_id, device_id, path_or_url, storage_type) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id.to_string())
        .bind(resource_id.to_string())
        .bind(&input.device_id)
        .bind(&input.path_or_url)
        .bind(encode_storage_type(&input.storage_type))
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(ResourceLocation {
            id,
            resource_id,
            device_id: input.device_id,
            path_or_url: input.path_or_url,
            storage_type: input.storage_type,
        })
    }

    async fn remove(&self, resource_id: Uuid, location_id: Uuid) -> Result<(), DomainError> {
        let affected =
            sqlx::query("DELETE FROM resource_locations WHERE resource_id = $1 AND id = $2")
                .bind(resource_id.to_string())
                .bind(location_id.to_string())
                .execute(&self.pool)
                .await
                .map_err(pg_err)?
                .rows_affected();

        if affected == 0 {
            return Err(DomainError::NotFound(format!(
                "location {location_id} not found for resource {resource_id}"
            )));
        }
        Ok(())
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgLocationRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl LocationRepository for PgLocationRepository {
    async fn list(&self, _resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn add(
        &self,
        _resource_id: Uuid,
        _input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn remove(&self, _resource_id: Uuid, _location_id: Uuid) -> Result<(), DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
