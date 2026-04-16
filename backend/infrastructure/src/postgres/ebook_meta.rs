use async_trait::async_trait;
use domain::{DomainError, EbookMeta, EbookMetaRepository, NewEbookMeta};
use uuid::Uuid;

#[derive(Default)]
pub struct PostgresEbookMetaRepository;

#[async_trait]
impl EbookMetaRepository for PostgresEbookMetaRepository {
    async fn get(&self, _resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres ebook meta repository is not implemented yet".to_string(),
        ))
    }

    async fn upsert(
        &self,
        _resource_id: Uuid,
        _input: NewEbookMeta,
    ) -> Result<EbookMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres ebook meta repository is not implemented yet".to_string(),
        ))
    }
}
