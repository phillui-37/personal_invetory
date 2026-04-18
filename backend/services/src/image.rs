use std::sync::Arc;

use domain::{
    DomainError, ImageMeta, ImageMetaRepository, LocationRepository, Resource, ResourceLocation,
    ResourceRepository,
};
use uuid::Uuid;

use crate::{map_validation_error, NewLocationInput};
use crate::{search::build_search_strategy, SearchConfig};
use use_cases::image::{NewImageInput, UpdateImageInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageDetail {
    pub resource: Resource,
    pub meta: ImageMeta,
    pub locations: Vec<ResourceLocation>,
}

pub struct ImageService {
    resource_repo: Arc<dyn ResourceRepository>,
    image_meta_repo: Arc<dyn ImageMetaRepository>,
    location_repo: Arc<dyn LocationRepository>,
    search_strategy: Arc<dyn domain::SearchStrategy>,
}

impl ImageService {
    pub fn new(
        resource_repo: Arc<dyn ResourceRepository>,
        image_meta_repo: Arc<dyn ImageMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
    ) -> Self {
        Self::new_with_search_config(
            resource_repo,
            image_meta_repo,
            location_repo,
            SearchConfig::default(),
        )
    }

    pub fn new_with_search_config(
        resource_repo: Arc<dyn ResourceRepository>,
        image_meta_repo: Arc<dyn ImageMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
        search_config: SearchConfig,
    ) -> Self {
        Self {
            resource_repo,
            image_meta_repo,
            location_repo,
            search_strategy: build_search_strategy(search_config),
        }
    }

    pub async fn list_images(&self) -> Result<Vec<Resource>, DomainError> {
        Ok(self
            .resource_repo
            .list()
            .await?
            .into_iter()
            .filter(|r| r.resource_type == domain::ResourceType::Image)
            .collect())
    }

    pub async fn search_images(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        Ok(self
            .search_strategy
            .search(self.resource_repo.as_ref(), query)
            .await?
            .into_iter()
            .filter(|r| r.resource_type == domain::ResourceType::Image)
            .collect())
    }

    pub async fn image_detail(&self, resource_id: Uuid) -> Result<ImageDetail, DomainError> {
        let resource = self.resource_repo.get_by_id(resource_id).await?;
        let meta = self.image_meta_repo.get(resource_id).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(ImageDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn add_image(&self, input: NewImageInput) -> Result<ImageDetail, DomainError> {
        let (resource_input, meta_input) =
            use_cases::image::validate_new_image(&input).map_err(map_validation_error)?;

        let resource = self.resource_repo.create(resource_input).await?;
        let meta = self.image_meta_repo.upsert(resource.id, meta_input).await?;
        let locations = self.location_repo.list(resource.id).await?;

        Ok(ImageDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn update_image(
        &self,
        resource_id: Uuid,
        input: UpdateImageInput,
    ) -> Result<ImageDetail, DomainError> {
        let existing = self.resource_repo.get_by_id(resource_id).await?;
        let existing_meta = self.image_meta_repo.get(resource_id).await?;
        let (resource_input, meta_input) =
            use_cases::image::validate_update_image(&existing, &existing_meta, &input)
                .map_err(map_validation_error)?;

        let resource = self
            .resource_repo
            .update(resource_id, resource_input)
            .await?;
        let meta = self.image_meta_repo.upsert(resource_id, meta_input).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(ImageDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn delete_image(&self, resource_id: Uuid) -> Result<(), DomainError> {
        self.resource_repo.delete(resource_id).await
    }

    pub async fn add_image_location(
        &self,
        resource_id: Uuid,
        input: NewLocationInput,
    ) -> Result<ResourceLocation, DomainError> {
        let location_input =
            use_cases::location::validate_new_location(&input).map_err(map_validation_error)?;
        self.location_repo.add(resource_id, location_input).await
    }

    pub async fn remove_image_location(
        &self,
        resource_id: Uuid,
        location_id: Uuid,
    ) -> Result<(), DomainError> {
        self.location_repo.remove(resource_id, location_id).await
    }
}
