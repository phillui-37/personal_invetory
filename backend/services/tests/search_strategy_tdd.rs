use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use domain::{
    DomainError, EbookMeta, EbookMetaRepository, LocationRepository, NewEbookMeta, NewResource,
    NewResourceLocation, Resource, ResourceLocation, ResourceRepository, ResourceType,
    UpdateResource,
};
use futures::executor::block_on;
use services::{EbookService, SearchConfig, SearchStrategyKind};
use uuid::Uuid;

#[derive(Default)]
struct FixtureResourceRepository {
    resources: Mutex<HashMap<Uuid, Resource>>,
    search_calls: Mutex<usize>,
    list_calls: Mutex<usize>,
}

impl FixtureResourceRepository {
    fn seed(&self, title: &str) {
        let id = Uuid::new_v4();
        let now = Utc::now();
        self.resources.lock().expect("resource lock").insert(
            id,
            Resource {
                id,
                title: title.to_string(),
                notes: None,
                resource_type: ResourceType::Ebook,
                created_at: now,
                updated_at: now,
            },
        );
    }

    fn search_call_count(&self) -> usize {
        *self.search_calls.lock().expect("search calls lock")
    }

    fn list_call_count(&self) -> usize {
        *self.list_calls.lock().expect("list calls lock")
    }
}

#[async_trait]
impl ResourceRepository for FixtureResourceRepository {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        *self.list_calls.lock().expect("list calls lock") += 1;
        Ok(self
            .resources
            .lock()
            .expect("resource lock")
            .values()
            .cloned()
            .collect())
    }

    async fn search(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        *self.search_calls.lock().expect("search calls lock") += 1;
        let query = query.to_lowercase();
        Ok(self
            .resources
            .lock()
            .expect("resource lock")
            .values()
            .filter(|resource| resource.title.to_lowercase().contains(&query))
            .cloned()
            .collect())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
        self.resources
            .lock()
            .expect("resource lock")
            .get(&id)
            .cloned()
            .ok_or_else(|| DomainError::NotFound(format!("resource {id} not found")))
    }

    async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let resource = Resource {
            id,
            title: input.title,
            notes: input.notes,
            resource_type: input.resource_type,
            created_at: now,
            updated_at: now,
        };
        self.resources
            .lock()
            .expect("resource lock")
            .insert(id, resource.clone());
        Ok(resource)
    }

    async fn update(&self, _id: Uuid, _input: UpdateResource) -> Result<Resource, DomainError> {
        Err(DomainError::InternalError("not needed".to_string()))
    }

    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }
}

#[derive(Default)]
struct NoopEbookMetaRepository;

#[async_trait]
impl EbookMetaRepository for NoopEbookMetaRepository {
    async fn get(&self, _resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        Err(DomainError::NotFound("not needed".to_string()))
    }

    async fn upsert(
        &self,
        _resource_id: Uuid,
        _input: NewEbookMeta,
    ) -> Result<EbookMeta, DomainError> {
        Err(DomainError::NotFound("not needed".to_string()))
    }
}

#[derive(Default)]
struct NoopLocationRepository;

#[async_trait]
impl LocationRepository for NoopLocationRepository {
    async fn list(&self, _resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        Ok(Vec::new())
    }

    async fn add(
        &self,
        _resource_id: Uuid,
        _input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError> {
        Err(DomainError::NotFound("not needed".to_string()))
    }

    async fn remove(&self, _resource_id: Uuid, _location_id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }
}

#[test]
fn default_search_strategy_falls_back_to_like() {
    let resource_repo = Arc::new(FixtureResourceRepository::default());
    resource_repo.seed("Rust Book");
    resource_repo.seed("Rst Bk Handbook");

    let service = EbookService::new(
        resource_repo.clone(),
        Arc::new(NoopEbookMetaRepository),
        Arc::new(NoopLocationRepository),
    );

    block_on(async {
        let result = service
            .search_ebooks("rust")
            .await
            .expect("default like search should work");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Rust Book");
    });

    assert_eq!(resource_repo.search_call_count(), 1);
    assert_eq!(resource_repo.list_call_count(), 0);
}

#[test]
fn fuzzy_strategy_differs_from_like_for_fixture_dataset() {
    let resource_repo = Arc::new(FixtureResourceRepository::default());
    resource_repo.seed("Rust Book");
    resource_repo.seed("Rusk Box");
    resource_repo.seed("Rocket Notes");

    let service = EbookService::new_with_search_config(
        resource_repo.clone(),
        Arc::new(NoopEbookMetaRepository),
        Arc::new(NoopLocationRepository),
        SearchConfig {
            strategy: SearchStrategyKind::Fuzzy,
        },
    );

    block_on(async {
        let result = service
            .search_ebooks("rsb")
            .await
            .expect("fuzzy search should return ranked results");
        let titles: Vec<String> = result.into_iter().map(|resource| resource.title).collect();
        assert_eq!(
            titles,
            vec!["Rusk Box".to_string(), "Rust Book".to_string()]
        );
    });

    assert_eq!(resource_repo.search_call_count(), 0);
    assert_eq!(resource_repo.list_call_count(), 1);
}
