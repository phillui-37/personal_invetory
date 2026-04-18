use std::sync::Arc;

use domain::{
    DomainError, LocationRepository, Resource, ResourceLocation, ResourceRepository, VideoMeta,
    VideoMetaRepository,
};
use uuid::Uuid;

use crate::{map_validation_error, NewLocationInput};
use crate::{search::build_search_strategy, SearchConfig};
use use_cases::video::{NewVideoInput, UpdateVideoInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoDetail {
    pub resource: Resource,
    pub meta: VideoMeta,
    pub locations: Vec<ResourceLocation>,
}

pub struct VideoService {
    resource_repo: Arc<dyn ResourceRepository>,
    video_meta_repo: Arc<dyn VideoMetaRepository>,
    location_repo: Arc<dyn LocationRepository>,
    search_strategy: Arc<dyn domain::SearchStrategy>,
}

impl VideoService {
    pub fn new(
        resource_repo: Arc<dyn ResourceRepository>,
        video_meta_repo: Arc<dyn VideoMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
    ) -> Self {
        Self::new_with_search_config(
            resource_repo,
            video_meta_repo,
            location_repo,
            SearchConfig::default(),
        )
    }

    pub fn new_with_search_config(
        resource_repo: Arc<dyn ResourceRepository>,
        video_meta_repo: Arc<dyn VideoMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
        search_config: SearchConfig,
    ) -> Self {
        Self {
            resource_repo,
            video_meta_repo,
            location_repo,
            search_strategy: build_search_strategy(search_config),
        }
    }

    pub async fn list_videos(&self) -> Result<Vec<Resource>, DomainError> {
        Ok(self
            .resource_repo
            .list()
            .await?
            .into_iter()
            .filter(|r| r.resource_type == domain::ResourceType::Video)
            .collect())
    }

    pub async fn search_videos(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        Ok(self
            .search_strategy
            .search(self.resource_repo.as_ref(), query)
            .await?
            .into_iter()
            .filter(|r| r.resource_type == domain::ResourceType::Video)
            .collect())
    }

    pub async fn video_detail(&self, resource_id: Uuid) -> Result<VideoDetail, DomainError> {
        let resource = self.resource_repo.get_by_id(resource_id).await?;
        let meta = self.video_meta_repo.get(resource_id).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(VideoDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn add_video(&self, input: NewVideoInput) -> Result<VideoDetail, DomainError> {
        let (resource_input, meta_input) =
            use_cases::video::validate_new_video(&input).map_err(map_validation_error)?;

        let resource = self.resource_repo.create(resource_input).await?;
        let meta = self.video_meta_repo.upsert(resource.id, meta_input).await?;
        let locations = self.location_repo.list(resource.id).await?;

        Ok(VideoDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn update_video(
        &self,
        resource_id: Uuid,
        input: UpdateVideoInput,
    ) -> Result<VideoDetail, DomainError> {
        let existing = self.resource_repo.get_by_id(resource_id).await?;
        let existing_meta = self.video_meta_repo.get(resource_id).await?;
        let (resource_input, meta_input) =
            use_cases::video::validate_update_video(&existing, &existing_meta, &input)
                .map_err(map_validation_error)?;

        let resource = self
            .resource_repo
            .update(resource_id, resource_input)
            .await?;
        let meta = self.video_meta_repo.upsert(resource_id, meta_input).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(VideoDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn delete_video(&self, resource_id: Uuid) -> Result<(), DomainError> {
        self.resource_repo.delete(resource_id).await
    }

    pub async fn add_video_location(
        &self,
        resource_id: Uuid,
        input: NewLocationInput,
    ) -> Result<ResourceLocation, DomainError> {
        let location_input =
            use_cases::location::validate_new_location(&input).map_err(map_validation_error)?;
        self.location_repo.add(resource_id, location_input).await
    }

    pub async fn remove_video_location(
        &self,
        resource_id: Uuid,
        location_id: Uuid,
    ) -> Result<(), DomainError> {
        self.location_repo.remove(resource_id, location_id).await
    }
}
