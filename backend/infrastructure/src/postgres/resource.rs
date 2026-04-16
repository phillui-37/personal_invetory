use async_trait::async_trait;
use domain::{DomainError, NewResource, Resource, ResourceRepository, UpdateResource};
use uuid::Uuid;

#[derive(Default)]
pub struct PostgresResourceRepository;

#[async_trait]
impl ResourceRepository for PostgresResourceRepository {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        Err(DomainError::InternalError(
            "postgres resource repository is not implemented yet".to_string(),
        ))
    }

    async fn search(&self, _query: &str) -> Result<Vec<Resource>, DomainError> {
        Err(DomainError::InternalError(
            "postgres resource repository is not implemented yet".to_string(),
        ))
    }

    async fn get_by_id(&self, _id: Uuid) -> Result<Resource, DomainError> {
        Err(DomainError::InternalError(
            "postgres resource repository is not implemented yet".to_string(),
        ))
    }

    async fn create(&self, _input: NewResource) -> Result<Resource, DomainError> {
        Err(DomainError::InternalError(
            "postgres resource repository is not implemented yet".to_string(),
        ))
    }

    async fn update(&self, _id: Uuid, _input: UpdateResource) -> Result<Resource, DomainError> {
        Err(DomainError::InternalError(
            "postgres resource repository is not implemented yet".to_string(),
        ))
    }

    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        Err(DomainError::InternalError(
            "postgres resource repository is not implemented yet".to_string(),
        ))
    }
}
