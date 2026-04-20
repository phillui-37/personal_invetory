use std::{collections::HashSet, sync::Arc};

use domain::{
    DomainError, GameMeta, GameMetaRepository, LocationRepository, Resource, ResourceLocation,
    ResourceRepository,
};
use uuid::Uuid;

use crate::{filter_resources_by_type, map_validation_error, NewLocationInput};
use crate::{search::build_search_strategy, SearchConfig};
use use_cases::game::{NewGameInput, UpdateGameInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameDetail {
    pub resource: Resource,
    pub meta: GameMeta,
    pub locations: Vec<ResourceLocation>,
}

pub struct GameService {
    resource_repo: Arc<dyn ResourceRepository>,
    game_meta_repo: Arc<dyn GameMetaRepository>,
    location_repo: Arc<dyn LocationRepository>,
    search_strategy: Arc<dyn domain::SearchStrategy>,
}

impl GameService {
    pub fn new(
        resource_repo: Arc<dyn ResourceRepository>,
        game_meta_repo: Arc<dyn GameMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
    ) -> Self {
        Self::new_with_search_config(
            resource_repo,
            game_meta_repo,
            location_repo,
            SearchConfig::default(),
        )
    }

    pub fn new_with_search_config(
        resource_repo: Arc<dyn ResourceRepository>,
        game_meta_repo: Arc<dyn GameMetaRepository>,
        location_repo: Arc<dyn LocationRepository>,
        search_config: SearchConfig,
    ) -> Self {
        Self {
            resource_repo,
            game_meta_repo,
            location_repo,
            search_strategy: build_search_strategy(search_config),
        }
    }

    pub async fn list_games(
        &self,
        allowed_ids: Option<&HashSet<Uuid>>,
    ) -> Result<Vec<Resource>, DomainError> {
        Ok(filter_resources_by_type(
            self.resource_repo.list().await?,
            domain::ResourceType::Game,
            allowed_ids,
        ))
    }

    pub async fn search_games(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        Ok(self
            .search_strategy
            .search(self.resource_repo.as_ref(), query)
            .await?
            .into_iter()
            .filter(|r| r.resource_type == domain::ResourceType::Game)
            .collect())
    }

    pub async fn game_detail(&self, resource_id: Uuid) -> Result<GameDetail, DomainError> {
        let resource = self.resource_repo.get_by_id(resource_id).await?;
        let meta = self.game_meta_repo.get(resource_id).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(GameDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn add_game(&self, input: NewGameInput) -> Result<GameDetail, DomainError> {
        let (resource_input, meta_input) =
            use_cases::game::validate_new_game(&input).map_err(map_validation_error)?;

        let resource = self.resource_repo.create(resource_input).await?;
        let meta = self.game_meta_repo.upsert(resource.id, meta_input).await?;
        let locations = self.location_repo.list(resource.id).await?;

        Ok(GameDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn update_game(
        &self,
        resource_id: Uuid,
        input: UpdateGameInput,
    ) -> Result<GameDetail, DomainError> {
        let existing = self.resource_repo.get_by_id(resource_id).await?;
        let existing_meta = self.game_meta_repo.get(resource_id).await?;
        let (resource_input, meta_input) =
            use_cases::game::validate_update_game(&existing, &existing_meta, &input)
                .map_err(map_validation_error)?;

        let resource = self
            .resource_repo
            .update(resource_id, resource_input)
            .await?;
        let meta = self.game_meta_repo.upsert(resource_id, meta_input).await?;
        let locations = self.location_repo.list(resource_id).await?;

        Ok(GameDetail {
            resource,
            meta,
            locations,
        })
    }

    pub async fn delete_game(&self, resource_id: Uuid) -> Result<(), DomainError> {
        self.resource_repo.delete(resource_id).await
    }

    pub async fn batch_update_games(
        &self,
        ids: Vec<Uuid>,
        input: UpdateGameInput,
    ) -> (usize, Vec<(Uuid, String)>) {
        let mut updated = 0usize;
        let mut failed = Vec::new();
        for id in ids {
            match self.update_game(id, input.clone()).await {
                Ok(_) => updated += 1,
                Err(e) => failed.push((id, format!("{e:?}"))),
            }
        }
        (updated, failed)
    }

    pub async fn batch_copy_game_meta(
        &self,
        source_id: Uuid,
        target_ids: Vec<Uuid>,
    ) -> (usize, Vec<(Uuid, String)>) {
        let source_meta = match self.game_meta_repo.get(source_id).await {
            Ok(m) => m,
            Err(e) => return (0, target_ids.into_iter().map(|id| (id, format!("{e:?}"))).collect()),
        };
        let input = UpdateGameInput {
            title: None,
            notes: None,
            platform: source_meta.platform.clone(),
            store: source_meta.store.clone(),
            developer: source_meta.developer.clone(),
            publisher: source_meta.publisher.clone(),
            manual_notes: source_meta.manual_notes.clone(),
        };
        let mut updated = 0usize;
        let mut failed = Vec::new();
        for id in target_ids {
            match self.update_game(id, input.clone()).await {
                Ok(_) => updated += 1,
                Err(e) => failed.push((id, format!("{e:?}"))),
            }
        }
        (updated, failed)
    }

    pub async fn add_game_location(
        &self,
        resource_id: Uuid,
        input: NewLocationInput,
    ) -> Result<ResourceLocation, DomainError> {
        let location_input =
            use_cases::location::validate_new_location(&input).map_err(map_validation_error)?;
        self.location_repo.add(resource_id, location_input).await
    }

    pub async fn remove_game_location(
        &self,
        resource_id: Uuid,
        location_id: Uuid,
    ) -> Result<(), DomainError> {
        self.location_repo.remove(resource_id, location_id).await
    }
}
