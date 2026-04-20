use std::{collections::HashSet, sync::Arc};

use domain::{
    DomainError, ImageMeta, ImageMetaRepository, LocationRepository, Resource, ResourceLocation,
    ResourceRepository,
};
use uuid::Uuid;

use crate::{filter_resources_by_type, map_validation_error, NewLocationInput};
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

    pub async fn list_images(
        &self,
        allowed_ids: Option<&HashSet<Uuid>>,
    ) -> Result<Vec<Resource>, DomainError> {
        Ok(filter_resources_by_type(
            self.resource_repo.list().await?,
            domain::ResourceType::Image,
            allowed_ids,
        ))
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

    pub async fn batch_update_images(
        &self,
        ids: Vec<Uuid>,
        input: UpdateImageInput,
    ) -> (usize, Vec<(Uuid, String)>) {
        let mut updated = 0usize;
        let mut failed = Vec::new();
        for id in ids {
            match self.update_image(id, input.clone()).await {
                Ok(_) => updated += 1,
                Err(e) => failed.push((id, format!("{e:?}"))),
            }
        }
        (updated, failed)
    }

    pub async fn batch_copy_image_meta(
        &self,
        source_id: Uuid,
        target_ids: Vec<Uuid>,
    ) -> (usize, Vec<(Uuid, String)>) {
        let source_meta = match self.image_meta_repo.get(source_id).await {
            Ok(m) => m,
            Err(e) => return (0, target_ids.into_iter().map(|id| (id, format!("{e:?}"))).collect()),
        };
        let input = UpdateImageInput {
            title: None,
            notes: None,
            width: source_meta.width,
            height: source_meta.height,
            file_format: source_meta.file_format.clone(),
            file_size_bytes: source_meta.file_size_bytes,
        };
        let mut updated = 0usize;
        let mut failed = Vec::new();
        for id in target_ids {
            match self.update_image(id, input.clone()).await {
                Ok(_) => updated += 1,
                Err(e) => failed.push((id, format!("{e:?}"))),
            }
        }
        (updated, failed)
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
