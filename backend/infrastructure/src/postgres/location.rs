use async_trait::async_trait;
use domain::{DomainError, LocationRepository, NewResourceLocation, ResourceLocation};
use uuid::Uuid;

#[derive(Default)]
pub struct PostgresLocationRepository;

#[async_trait]
impl LocationRepository for PostgresLocationRepository {
    async fn list(&self, _resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        Err(DomainError::InternalError(
            "postgres location repository is not implemented yet".to_string(),
        ))
    }

    async fn add(
        &self,
        _resource_id: Uuid,
        _input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError> {
        Err(DomainError::InternalError(
            "postgres location repository is not implemented yet".to_string(),
        ))
    }

    async fn remove(&self, _resource_id: Uuid, _location_id: Uuid) -> Result<(), DomainError> {
        Err(DomainError::InternalError(
            "postgres location repository is not implemented yet".to_string(),
        ))
    }
}
