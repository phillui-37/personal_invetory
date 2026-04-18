use async_trait::async_trait;
use domain::{ChapterCheck, ChapterCheckRepository, DomainError};
use uuid::Uuid;

#[derive(Default)]
pub struct PostgresChapterCheckRepository;

#[async_trait]
impl ChapterCheckRepository for PostgresChapterCheckRepository {
    async fn create(
        &self,
        _resource_id: Uuid,
        _has_new_chapter: bool,
        _latest_chapter: Option<String>,
        _error_message: Option<String>,
    ) -> Result<ChapterCheck, DomainError> {
        Err(DomainError::InternalError(
            "postgres chapter check repository is not implemented yet".to_string(),
        ))
    }

    async fn list(&self, _resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError> {
        Err(DomainError::InternalError(
            "postgres chapter check repository is not implemented yet".to_string(),
        ))
    }
}
