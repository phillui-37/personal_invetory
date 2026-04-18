use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use services::{NewEbookInput, NewLocationInput};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchImportLocationInput {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchImportEbookEntry {
    pub title: String,
    pub notes: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
    #[serde(default)]
    pub locations: Vec<BatchImportLocationInput>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchImportSuccess {
    pub index: usize,
    pub resource_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchImportFailure {
    pub index: usize,
    pub error: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchImportResponse {
    pub succeeded: Vec<BatchImportSuccess>,
    pub failed: Vec<BatchImportFailure>,
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/ebooks/batch-import",
    request_body = Vec<BatchImportEbookEntry>,
    responses(
        (status = 200, description = "Batch import result", body = BatchImportResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn batch_import_ebooks(
    State(state): State<Arc<AppState>>,
    Json(entries): Json<Vec<BatchImportEbookEntry>>,
) -> Result<Json<BatchImportResponse>, ApiError> {
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for (index, entry) in entries.into_iter().enumerate() {
        if entry.title.trim().is_empty() {
            failed.push(BatchImportFailure {
                index,
                error: "title must not be empty".to_string(),
            });
            continue;
        }

        let detail = match state
            .ebook_service
            .add_ebook(NewEbookInput {
                title: entry.title,
                notes: entry.notes,
                author: entry.author,
                isbn: entry.isbn,
                publisher: entry.publisher,
                language: entry.language,
                file_format: entry.file_format,
            })
            .await
        {
            Ok(detail) => detail,
            Err(e) => {
                failed.push(BatchImportFailure {
                    index,
                    error: format!("{e:?}"),
                });
                continue;
            }
        };

        let resource_id = detail.resource.id;
        let mut location_error = None;

        for loc in entry.locations {
            if let Err(e) = state
                .ebook_service
                .add_ebook_location(
                    resource_id,
                    NewLocationInput {
                        device_id: loc.device_id,
                        path_or_url: loc.path_or_url,
                        storage_type: loc.storage_type,
                    },
                )
                .await
            {
                eprintln!("batch import: location add failed for {resource_id}: {e:?}");
                location_error = Some(format!("{e:?}"));
                break;
            }
        }

        if let Some(error) = location_error {
            failed.push(BatchImportFailure { index, error });
            continue;
        }

        succeeded.push(BatchImportSuccess { index, resource_id });
    }

    Ok(Json(BatchImportResponse { succeeded, failed }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{extract::State, Json};
    use chrono::Utc;
    use domain::{
        DomainError, EbookMeta, EbookMetaRepository, LocationRepository, NewEbookMeta,
        NewResource, NewResourceLocation, Resource, ResourceLocation, ResourceRepository,
        ResourceType, UpdateResource,
    };
    use services::EbookService;
    use uuid::Uuid;

    use super::{batch_import_ebooks, BatchImportEbookEntry, BatchImportLocationInput};
    use crate::state::AppState;

    #[derive(Default)]
    struct TestResourceRepository;

    #[async_trait]
    impl ResourceRepository for TestResourceRepository {
        async fn list(&self) -> Result<Vec<Resource>, DomainError> {
            Ok(Vec::new())
        }

        async fn search(&self, _query: &str) -> Result<Vec<Resource>, DomainError> {
            Ok(Vec::new())
        }

        async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
            Err(DomainError::NotFound(format!("resource {id} not found")))
        }

        async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
            let now = Utc::now();
            Ok(Resource {
                id: Uuid::new_v4(),
                title: input.title,
                notes: input.notes,
                resource_type: ResourceType::Ebook,
                created_at: now,
                updated_at: now,
            })
        }

        async fn update(&self, id: Uuid, _input: UpdateResource) -> Result<Resource, DomainError> {
            Err(DomainError::NotFound(format!("resource {id} not found")))
        }

        async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct TestEbookMetaRepository;

    #[async_trait]
    impl EbookMetaRepository for TestEbookMetaRepository {
        async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError> {
            Err(DomainError::NotFound(format!(
                "ebook meta for {resource_id} not found"
            )))
        }

        async fn upsert(
            &self,
            resource_id: Uuid,
            input: NewEbookMeta,
        ) -> Result<EbookMeta, DomainError> {
            Ok(EbookMeta {
                resource_id,
                author: input.author,
                isbn: input.isbn,
                publisher: input.publisher,
                language: input.language,
                file_format: input.file_format,
            })
        }
    }

    #[derive(Default)]
    struct FailingLocationRepository;

    #[async_trait]
    impl LocationRepository for FailingLocationRepository {
        async fn list(&self, _resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
            Ok(Vec::new())
        }

        async fn add(
            &self,
            _resource_id: Uuid,
            _input: NewResourceLocation,
        ) -> Result<ResourceLocation, DomainError> {
            Err(DomainError::InternalError(
                "location repository write failed".to_string(),
            ))
        }

        async fn remove(&self, _resource_id: Uuid, _location_id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn batch_import_reports_location_failures_as_failed_entries() {
        let ebook_service = Arc::new(EbookService::new(
            Arc::new(TestResourceRepository),
            Arc::new(TestEbookMetaRepository),
            Arc::new(FailingLocationRepository),
        ));
        let mut state = AppState::for_tests("secret-key".to_string());
        state.ebook_service = ebook_service;

        let response = batch_import_ebooks(
            State(Arc::new(state)),
            Json(vec![BatchImportEbookEntry {
                title: "Book A".to_string(),
                notes: None,
                author: None,
                isbn: None,
                publisher: None,
                language: None,
                file_format: Some("epub".to_string()),
                locations: vec![BatchImportLocationInput {
                    device_id: "device-1".to_string(),
                    path_or_url: "/books/book-a.epub".to_string(),
                    storage_type: "LocalFs".to_string(),
                }],
            }]),
        )
        .await
        .expect("batch import response")
        .0;

        assert!(response.succeeded.is_empty());
        assert_eq!(response.failed.len(), 1);
        assert_eq!(response.failed[0].index, 0);
        assert!(response.failed[0].error.contains("location repository write failed"));
    }
}
