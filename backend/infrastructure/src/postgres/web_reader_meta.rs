use async_trait::async_trait;
use domain::{DomainError, NewWebReaderMeta, WebReaderMeta, WebReaderMetaRepository};
use uuid::Uuid;

#[derive(Default)]
pub struct PostgresWebReaderMetaRepository;

#[async_trait]
impl WebReaderMetaRepository for PostgresWebReaderMetaRepository {
    async fn get(&self, _resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres web reader meta repository is not implemented yet".to_string(),
        ))
    }

    async fn upsert(
        &self,
        _resource_id: Uuid,
        _input: NewWebReaderMeta,
    ) -> Result<WebReaderMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres web reader meta repository is not implemented yet".to_string(),
        ))
    }
}
