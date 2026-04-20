use std::{collections::HashSet, sync::Arc};

use domain::{
    DomainError, LocationRepository, Resource, ResourceLocation, ResourceRepository,
    WebReaderMeta, WebReaderMetaRepository,
};
use uuid::Uuid;

use crate::{filter_resources_by_type, map_validation_error, NewLocationInput, NewWebReaderInput, UpdateWebReaderInput};
use crate::{search::build_search_strategy, SearchConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebReaderDetail {
    pub resource: Resource,
    pub meta: WebReaderMeta,
    pub locations: Vec<ResourceLocation>,
}

pub struct WebReaderService {
    resource_repo: Arc<dyn ResourceRepository>,
    web_reader_meta_repo: Arc<dyn WebReaderMetaRepository>,
    location_repo: Arc<dyn LocationRepository>,
    search_strategy: Arc<dyn domain::SearchStrategy>,
}

impl WebReaderService {
    pub fn new(
        resource_repo: Arc<dyn ResourceRepository>,
        web_reader_meta_repo: Arc<dyn WebReaderMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
    ) -> Self {
        Self::new_with_search_config(
            resource_repo,
            web_reader_meta_repo,
            location_repo,
            SearchConfig::default(),
        )
    }

    pub fn new_with_search_config(
        resource_repo: Arc<dyn ResourceRepository>,
        web_reader_meta_repo: Arc<dyn WebReaderMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
        search_config: SearchConfig,
    ) -> Self {
        Self {
            resource_repo,
            web_reader_meta_repo,
            location_repo,
            search_strategy: build_search_strategy(search_config),
        }
    }

    pub async fn list_web_readers(
        &self,
        allowed_ids: Option<&HashSet<Uuid>>,
    ) -> Result<Vec<Resource>, DomainError> {
        Ok(filter_resources_by_type(
            self.resource_repo.list().await?,
            domain::ResourceType::WebReader,
            allowed_ids,
        ))
    }

    pub async fn search_web_readers(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        Ok(self
            .search_strategy
            .search(self.resource_repo.as_ref(), query)
            .await?
            .into_iter()
            .filter(|resource| resource.resource_type == domain::ResourceType::WebReader)
            .collect())
    }

    pub async fn web_reader_detail(
        &self,
        resource_id: Uuid,
    ) -> Result<WebReaderDetail, DomainError> {
        let resource = self.resource_repo.get_by_id(resource_id).await?;
        let meta = self.web_reader_meta_repo.get(resource_id).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(WebReaderDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn add_web_reader(
        &self,
        input: NewWebReaderInput,
    ) -> Result<WebReaderDetail, DomainError> {
        let (resource_input, meta_input) =
            use_cases::web_reader::validate_new_web_reader(&input).map_err(map_validation_error)?;

        let resource = self.resource_repo.create(resource_input).await?;
        let meta = self
            .web_reader_meta_repo
            .upsert(resource.id, meta_input)
            .await?;
        let locations = self.location_repo.list(resource.id).await?;

        Ok(WebReaderDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn update_web_reader(
        &self,
        resource_id: Uuid,
        input: UpdateWebReaderInput,
    ) -> Result<WebReaderDetail, DomainError> {
        let existing = self.resource_repo.get_by_id(resource_id).await?;
        let existing_meta = self.web_reader_meta_repo.get(resource_id).await?;
        let (resource_input, meta_input) =
            use_cases::web_reader::validate_update_web_reader(&existing, &existing_meta, &input)
                .map_err(map_validation_error)?;

        let resource = self
            .resource_repo
            .update(resource_id, resource_input)
            .await?;
        let meta = self
            .web_reader_meta_repo
            .upsert(resource_id, meta_input)
            .await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(WebReaderDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn delete_web_reader(&self, resource_id: Uuid) -> Result<(), DomainError> {
        self.resource_repo.delete(resource_id).await
    }

    pub async fn add_web_reader_location(
        &self,
        resource_id: Uuid,
        input: NewLocationInput,
    ) -> Result<ResourceLocation, DomainError> {
        let location_input =
            use_cases::location::validate_new_location(&input).map_err(map_validation_error)?;
        self.location_repo.add(resource_id, location_input).await
    }

    pub async fn remove_web_reader_location(
        &self,
        resource_id: Uuid,
        location_id: Uuid,
    ) -> Result<(), DomainError> {
        self.location_repo.remove(resource_id, location_id).await
    }
}
