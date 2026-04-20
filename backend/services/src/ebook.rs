use std::{collections::HashSet, sync::Arc};

use domain::{
    DomainError, EbookMeta, EbookMetaRepository, LocationRepository, Resource, ResourceLocation,
    ResourceRepository,
};
use uuid::Uuid;

use crate::{filter_resources_by_type, map_validation_error, NewEbookInput, NewLocationInput, UpdateEbookInput};
use crate::{search::build_search_strategy, SearchConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EbookDetail {
    pub resource: Resource,
    pub meta: EbookMeta,
    pub locations: Vec<ResourceLocation>,
}

pub struct EbookService {
    resource_repo: Arc<dyn ResourceRepository>,
    ebook_meta_repo: Arc<dyn EbookMetaRepository>,
    location_repo: Arc<dyn LocationRepository>,
    search_strategy: Arc<dyn domain::SearchStrategy>,
}

impl EbookService {
    pub fn new(
        resource_repo: Arc<dyn ResourceRepository>,
        ebook_meta_repo: Arc<dyn EbookMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
    ) -> Self {
        Self::new_with_search_config(
            resource_repo,
            ebook_meta_repo,
            location_repo,
            SearchConfig::default(),
        )
    }

    pub fn new_with_search_config(
        resource_repo: Arc<dyn ResourceRepository>,
        ebook_meta_repo: Arc<dyn EbookMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
        search_config: SearchConfig,
    ) -> Self {
        Self {
            resource_repo,
            ebook_meta_repo,
            location_repo,
            search_strategy: build_search_strategy(search_config),
        }
    }

    pub async fn list_ebooks(
        &self,
        allowed_ids: Option<&HashSet<Uuid>>,
    ) -> Result<Vec<Resource>, DomainError> {
        Ok(filter_resources_by_type(
            self.resource_repo.list().await?,
            domain::ResourceType::Ebook,
            allowed_ids,
        ))
    }

    pub async fn search_ebooks(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        Ok(self
            .search_strategy
            .search(self.resource_repo.as_ref(), query)
            .await?
            .into_iter()
            .filter(|r| r.resource_type == domain::ResourceType::Ebook)
            .collect())
    }

    pub async fn ebook_detail(&self, resource_id: Uuid) -> Result<EbookDetail, DomainError> {
        let resource = self.resource_repo.get_by_id(resource_id).await?;
        let meta = self.ebook_meta_repo.get(resource_id).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(EbookDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn add_ebook(&self, input: NewEbookInput) -> Result<EbookDetail, DomainError> {
        let (resource_input, meta_input) =
            use_cases::ebook::validate_new_ebook(&input).map_err(map_validation_error)?;

        let resource = self.resource_repo.create(resource_input).await?;
        let meta = self.ebook_meta_repo.upsert(resource.id, meta_input).await?;
        let locations = self.location_repo.list(resource.id).await?;

        Ok(EbookDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn update_ebook(
        &self,
        resource_id: Uuid,
        input: UpdateEbookInput,
    ) -> Result<EbookDetail, DomainError> {
        let existing = self.resource_repo.get_by_id(resource_id).await?;
        let existing_meta = self.ebook_meta_repo.get(resource_id).await?;
        let (resource_input, meta_input) =
            use_cases::ebook::validate_update_ebook(&existing, &existing_meta, &input)
                .map_err(map_validation_error)?;

        let resource = self
            .resource_repo
            .update(resource_id, resource_input)
            .await?;
        let meta = self.ebook_meta_repo.upsert(resource_id, meta_input).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(EbookDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn delete_ebook(&self, resource_id: Uuid) -> Result<(), DomainError> {
        self.resource_repo.delete(resource_id).await
    }

    pub async fn add_ebook_location(
        &self,
        resource_id: Uuid,
        input: NewLocationInput,
    ) -> Result<ResourceLocation, DomainError> {
        let location_input =
            use_cases::location::validate_new_location(&input).map_err(map_validation_error)?;
        self.location_repo.add(resource_id, location_input).await
    }

    pub async fn remove_ebook_location(
        &self,
        resource_id: Uuid,
        location_id: Uuid,
    ) -> Result<(), DomainError> {
        self.location_repo.remove(resource_id, location_id).await
    }
}
