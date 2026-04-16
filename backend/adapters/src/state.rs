use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use domain::{
    DomainError, EbookMeta, EbookMetaRepository, LocationRepository, NewEbookMeta, NewResource,
    NewResourceLocation, NewWebReaderMeta, Resource, ResourceLocation, ResourceRepository,
    UpdateResource, WebReaderMeta, WebReaderMetaRepository,
};
use plugins::PluginRegistry;
use services::{EbookService, WebReaderService};
use uuid::Uuid;

pub struct AppState {
    pub ebook_service: Arc<EbookService>,
    pub web_reader_service: Arc<WebReaderService>,
    pub plugin_registry: Arc<PluginRegistry>,
    pub api_key: String,
    pub openapi_json: String,
}

impl AppState {
    pub fn new(
        ebook_service: Arc<EbookService>,
        web_reader_service: Arc<WebReaderService>,
        api_key: String,
    ) -> Self {
        Self {
            ebook_service,
            web_reader_service,
            plugin_registry: Arc::new(PluginRegistry::default()),
            api_key,
            openapi_json: "{}".to_string(),
        }
    }

    pub fn for_tests(api_key: String) -> Self {
        let resource_repo = Arc::new(NoopResourceRepository::default());
        let location_repo = Arc::new(NoopLocationRepository::default());

        let ebook_service = Arc::new(EbookService::new(
            resource_repo.clone(),
            Arc::new(NoopEbookMetaRepository::default()),
            location_repo.clone(),
        ));
        let web_reader_service = Arc::new(WebReaderService::new(
            resource_repo,
            Arc::new(NoopWebReaderMetaRepository::default()),
            location_repo,
        ));

        Self::new(ebook_service, web_reader_service, api_key)
    }
}

#[derive(Default)]
struct NoopResourceRepository;

#[async_trait]
impl ResourceRepository for NoopResourceRepository {
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
            resource_type: input.resource_type,
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
struct NoopEbookMetaRepository;

#[async_trait]
impl EbookMetaRepository for NoopEbookMetaRepository {
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
struct NoopWebReaderMetaRepository;

#[async_trait]
impl WebReaderMetaRepository for NoopWebReaderMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
        Err(DomainError::NotFound(format!(
            "web reader meta for {resource_id} not found"
        )))
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewWebReaderMeta,
    ) -> Result<WebReaderMeta, DomainError> {
        Ok(WebReaderMeta {
            resource_id,
            url: input.url,
            site_name: input.site_name,
            last_checked_chapter: input.last_checked_chapter,
        })
    }
}

#[derive(Default)]
struct NoopLocationRepository {
    _keep: HashMap<Uuid, Vec<ResourceLocation>>,
}

#[async_trait]
impl LocationRepository for NoopLocationRepository {
    async fn list(&self, _resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        Ok(Vec::new())
    }

    async fn add(
        &self,
        resource_id: Uuid,
        input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError> {
        Ok(ResourceLocation {
            id: Uuid::new_v4(),
            resource_id,
            device_id: input.device_id,
            path_or_url: input.path_or_url,
            storage_type: input.storage_type,
        })
    }

    async fn remove(&self, _resource_id: Uuid, _location_id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }
}
