use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use domain::{
    DomainError, EbookMeta, EbookMetaRepository, LocationRepository, NewEbookMeta, NewResource,
    NewResourceLocation, NewWebReaderMeta, Resource, ResourceLocation, ResourceRepository,
    ResourceType, StorageType, UpdateResource, WebReaderMeta, WebReaderMetaRepository,
};
use futures::executor::block_on;
use services::{
    EbookService, NewEbookInput, NewLocationInput, NewWebReaderInput, UpdateEbookInput,
    WebReaderService,
};
use uuid::Uuid;

#[derive(Default)]
struct MockResourceRepository {
    resources: Mutex<HashMap<Uuid, Resource>>,
    fail: Mutex<HashMap<&'static str, DomainError>>,
    create_calls: Mutex<usize>,
}

impl MockResourceRepository {
    fn fail_once(&self, method: &'static str, error: DomainError) {
        self.fail
            .lock()
            .expect("resource fail lock")
            .insert(method, error);
    }

    fn create_call_count(&self) -> usize {
        *self.create_calls.lock().expect("create calls lock")
    }

    fn consume_fail(&self, method: &'static str) -> Option<DomainError> {
        self.fail.lock().expect("resource fail lock").remove(method)
    }
}

#[async_trait]
impl ResourceRepository for MockResourceRepository {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        if let Some(error) = self.consume_fail("list") {
            return Err(error);
        }

        Ok(self
            .resources
            .lock()
            .expect("resource lock")
            .values()
            .cloned()
            .collect())
    }

    async fn search(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
        if let Some(error) = self.consume_fail("search") {
            return Err(error);
        }

        Ok(self
            .resources
            .lock()
            .expect("resource lock")
            .values()
            .filter(|resource| {
                resource
                    .title
                    .to_lowercase()
                    .contains(&query.to_lowercase())
            })
            .cloned()
            .collect())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
        if let Some(error) = self.consume_fail("get_by_id") {
            return Err(error);
        }

        self.resources
            .lock()
            .expect("resource lock")
            .get(&id)
            .cloned()
            .ok_or_else(|| DomainError::NotFound(format!("resource {id} not found")))
    }

    async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
        if let Some(error) = self.consume_fail("create") {
            return Err(error);
        }

        *self.create_calls.lock().expect("create calls lock") += 1;
        let now = Utc::now();
        let resource = Resource {
            id: Uuid::new_v4(),
            title: input.title,
            notes: input.notes,
            resource_type: input.resource_type,
            created_at: now,
            updated_at: now,
        };
        self.resources
            .lock()
            .expect("resource lock")
            .insert(resource.id, resource.clone());
        Ok(resource)
    }

    async fn update(&self, id: Uuid, input: UpdateResource) -> Result<Resource, DomainError> {
        if let Some(error) = self.consume_fail("update") {
            return Err(error);
        }

        let mut guard = self.resources.lock().expect("resource lock");
        let resource = guard
            .get_mut(&id)
            .ok_or_else(|| DomainError::NotFound(format!("resource {id} not found")))?;

        if let Some(title) = input.title {
            resource.title = title;
        }
        if let Some(notes) = input.notes {
            resource.notes = Some(notes);
        }
        resource.updated_at = Utc::now();
        Ok(resource.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        if let Some(error) = self.consume_fail("delete") {
            return Err(error);
        }

        let removed = self.resources.lock().expect("resource lock").remove(&id);
        if removed.is_none() {
            return Err(DomainError::NotFound(format!("resource {id} not found")));
        }
        Ok(())
    }
}

#[derive(Default)]
struct MockEbookMetaRepository {
    metas: Mutex<HashMap<Uuid, EbookMeta>>,
    fail: Mutex<HashMap<&'static str, DomainError>>,
}

impl MockEbookMetaRepository {
    fn consume_fail(&self, method: &'static str) -> Option<DomainError> {
        self.fail
            .lock()
            .expect("ebook meta fail lock")
            .remove(method)
    }
}

#[async_trait]
impl EbookMetaRepository for MockEbookMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        if let Some(error) = self.consume_fail("get") {
            return Err(error);
        }

        self.metas
            .lock()
            .expect("ebook meta lock")
            .get(&resource_id)
            .cloned()
            .ok_or_else(|| DomainError::NotFound(format!("ebook meta {resource_id} not found")))
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewEbookMeta,
    ) -> Result<EbookMeta, DomainError> {
        if let Some(error) = self.consume_fail("upsert") {
            return Err(error);
        }

        let meta = EbookMeta {
            resource_id,
            author: input.author,
            isbn: input.isbn,
            publisher: input.publisher,
            language: input.language,
            file_format: input.file_format,
        };
        self.metas
            .lock()
            .expect("ebook meta lock")
            .insert(resource_id, meta.clone());
        Ok(meta)
    }
}

#[derive(Default)]
struct MockWebReaderMetaRepository {
    metas: Mutex<HashMap<Uuid, WebReaderMeta>>,
    fail: Mutex<HashMap<&'static str, DomainError>>,
}

impl MockWebReaderMetaRepository {
    fn consume_fail(&self, method: &'static str) -> Option<DomainError> {
        self.fail
            .lock()
            .expect("web reader meta fail lock")
            .remove(method)
    }
}

#[async_trait]
impl WebReaderMetaRepository for MockWebReaderMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
        if let Some(error) = self.consume_fail("get") {
            return Err(error);
        }

        self.metas
            .lock()
            .expect("web reader meta lock")
            .get(&resource_id)
            .cloned()
            .ok_or_else(|| {
                DomainError::NotFound(format!("web reader meta {resource_id} not found"))
            })
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewWebReaderMeta,
    ) -> Result<WebReaderMeta, DomainError> {
        if let Some(error) = self.consume_fail("upsert") {
            return Err(error);
        }

        let meta = WebReaderMeta {
            resource_id,
            url: input.url,
            site_name: input.site_name,
            last_checked_chapter: input.last_checked_chapter,
        };
        self.metas
            .lock()
            .expect("web reader meta lock")
            .insert(resource_id, meta.clone());
        Ok(meta)
    }
}

#[derive(Default)]
struct MockLocationRepository {
    locations: Mutex<HashMap<Uuid, Vec<ResourceLocation>>>,
    fail: Mutex<HashMap<&'static str, DomainError>>,
}

impl MockLocationRepository {
    fn fail_once(&self, method: &'static str, error: DomainError) {
        self.fail
            .lock()
            .expect("location fail lock")
            .insert(method, error);
    }

    fn consume_fail(&self, method: &'static str) -> Option<DomainError> {
        self.fail.lock().expect("location fail lock").remove(method)
    }
}

#[async_trait]
impl LocationRepository for MockLocationRepository {
    async fn list(&self, resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        if let Some(error) = self.consume_fail("list") {
            return Err(error);
        }

        Ok(self
            .locations
            .lock()
            .expect("location lock")
            .get(&resource_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn add(
        &self,
        resource_id: Uuid,
        input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError> {
        if let Some(error) = self.consume_fail("add") {
            return Err(error);
        }

        let location = ResourceLocation {
            id: Uuid::new_v4(),
            resource_id,
            device_id: input.device_id,
            path_or_url: input.path_or_url,
            storage_type: input.storage_type,
        };
        self.locations
            .lock()
            .expect("location lock")
            .entry(resource_id)
            .or_default()
            .push(location.clone());
        Ok(location)
    }

    async fn remove(&self, resource_id: Uuid, location_id: Uuid) -> Result<(), DomainError> {
        if let Some(error) = self.consume_fail("remove") {
            return Err(error);
        }

        let mut guard = self.locations.lock().expect("location lock");
        let locations = guard.get_mut(&resource_id).ok_or_else(|| {
            DomainError::NotFound(format!("resource {resource_id} has no locations"))
        })?;
        let initial_len = locations.len();
        locations.retain(|location| location.id != location_id);
        if locations.len() == initial_len {
            return Err(DomainError::NotFound(format!(
                "location {location_id} not found for resource {resource_id}"
            )));
        }
        Ok(())
    }
}

fn seed_ebook_resource(repo: &MockResourceRepository) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    repo.resources.lock().expect("resource lock").insert(
        id,
        Resource {
            id,
            title: "Seed Ebook".to_string(),
            notes: None,
            resource_type: ResourceType::Ebook,
            created_at: now,
            updated_at: now,
        },
    );
    id
}

fn seed_web_reader_resource(repo: &MockResourceRepository) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    repo.resources.lock().expect("resource lock").insert(
        id,
        Resource {
            id,
            title: "Seed Reader".to_string(),
            notes: None,
            resource_type: ResourceType::WebReader,
            created_at: now,
            updated_at: now,
        },
    );
    id
}

#[test]
fn ebook_happy_path_adds_resource_and_meta() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let ebook_meta_repo = Arc::new(MockEbookMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());

    let service = EbookService::new(resource_repo.clone(), ebook_meta_repo, location_repo);

    let input = NewEbookInput {
        title: "The Rust Book".to_string(),
        notes: Some("notes".to_string()),
        author: Some("Steve".to_string()),
        isbn: None,
        publisher: None,
        language: Some("en".to_string()),
        file_format: Some("EPUB".to_string()),
    };

    block_on(async {
        let detail = service.add_ebook(input).await.expect("add ebook");
        assert_eq!(detail.resource.title, "The Rust Book");
        assert_eq!(detail.resource.resource_type, ResourceType::Ebook);
        assert_eq!(detail.meta.file_format.as_deref(), Some("epub"));
        assert!(detail.locations.is_empty());
    });
}

#[test]
fn ebook_validation_error_maps_and_skips_repo_calls() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let ebook_meta_repo = Arc::new(MockEbookMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());

    let service = EbookService::new(resource_repo.clone(), ebook_meta_repo, location_repo);

    let input = NewEbookInput {
        title: " ".to_string(),
        notes: None,
        author: None,
        isbn: None,
        publisher: None,
        language: None,
        file_format: Some("epub".to_string()),
    };

    block_on(async {
        let error = service
            .add_ebook(input)
            .await
            .expect_err("validation should fail");
        match error {
            DomainError::ValidationError(message) => assert!(message.contains("title")),
            _ => panic!("expected validation error"),
        }
    });

    assert_eq!(resource_repo.create_call_count(), 0);
}

#[test]
fn ebook_propagates_conflict_error_from_repo() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    resource_repo.fail_once(
        "create",
        DomainError::Conflict("duplicate title".to_string()),
    );
    let ebook_meta_repo = Arc::new(MockEbookMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());

    let service = EbookService::new(resource_repo, ebook_meta_repo, location_repo);

    let input = NewEbookInput {
        title: "Book".to_string(),
        notes: None,
        author: None,
        isbn: None,
        publisher: None,
        language: None,
        file_format: Some("pdf".to_string()),
    };

    block_on(async {
        let error = service
            .add_ebook(input)
            .await
            .expect_err("repo conflict should propagate");
        assert!(matches!(error, DomainError::Conflict(_)));
    });
}

#[test]
fn ebook_propagates_not_found_error_from_repo() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    resource_repo.fail_once("get_by_id", DomainError::NotFound("missing".to_string()));
    let ebook_meta_repo = Arc::new(MockEbookMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());

    let service = EbookService::new(resource_repo, ebook_meta_repo, location_repo);

    let input = UpdateEbookInput {
        title: Some("Updated".to_string()),
        notes: None,
        author: None,
        isbn: None,
        publisher: None,
        language: None,
        file_format: Some("mobi".to_string()),
    };

    block_on(async {
        let error = service
            .update_ebook(Uuid::new_v4(), input)
            .await
            .expect_err("missing should propagate");
        assert!(matches!(error, DomainError::NotFound(_)));
    });
}

#[test]
fn ebook_propagates_internal_error_from_repo() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let resource_id = seed_ebook_resource(&resource_repo);
    let ebook_meta_repo = Arc::new(MockEbookMetaRepository::default());
    ebook_meta_repo
        .metas
        .lock()
        .expect("ebook meta lock")
        .insert(
            resource_id,
            EbookMeta {
                resource_id,
                author: None,
                isbn: None,
                publisher: None,
                language: None,
                file_format: Some("pdf".to_string()),
            },
        );
    let location_repo = Arc::new(MockLocationRepository::default());
    location_repo.fail_once("list", DomainError::InternalError("db offline".to_string()));

    let service = EbookService::new(resource_repo, ebook_meta_repo, location_repo);

    block_on(async {
        let error = service
            .ebook_detail(resource_id)
            .await
            .expect_err("internal error should propagate");
        assert!(matches!(error, DomainError::InternalError(_)));
    });
}

#[test]
fn ebook_location_validation_maps_to_domain_validation_error() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let ebook_meta_repo = Arc::new(MockEbookMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());
    let service = EbookService::new(resource_repo, ebook_meta_repo, location_repo);

    let location_input = NewLocationInput {
        device_id: "d1".to_string(),
        path_or_url: "/path".to_string(),
        storage_type: "unknown".to_string(),
    };

    block_on(async {
        let error = service
            .add_ebook_location(Uuid::new_v4(), location_input)
            .await
            .expect_err("location validation should fail");
        assert!(matches!(error, DomainError::ValidationError(_)));
    });
}

#[test]
fn web_reader_happy_path_adds_resource_and_meta() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let web_reader_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());
    let service = WebReaderService::new(resource_repo, web_reader_meta_repo, location_repo);

    let input = NewWebReaderInput {
        title: "Reader".to_string(),
        notes: Some("note".to_string()),
        url: "https://example.com/chapter/1".to_string(),
        site_name: Some("Example".to_string()),
        last_checked_chapter: Some("10".to_string()),
    };

    block_on(async {
        let detail = service.add_web_reader(input).await.expect("add web reader");
        assert_eq!(detail.resource.resource_type, ResourceType::WebReader);
        assert_eq!(detail.meta.url, "https://example.com/chapter/1");
    });
}

#[test]
fn web_reader_validation_error_maps_and_skips_repo_calls() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let web_reader_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());
    let service = WebReaderService::new(resource_repo.clone(), web_reader_meta_repo, location_repo);

    let input = NewWebReaderInput {
        title: "Reader".to_string(),
        notes: None,
        url: "not-a-url".to_string(),
        site_name: None,
        last_checked_chapter: None,
    };

    block_on(async {
        let error = service
            .add_web_reader(input)
            .await
            .expect_err("validation should fail");
        assert!(matches!(error, DomainError::ValidationError(_)));
    });

    assert_eq!(resource_repo.create_call_count(), 0);
}

#[test]
fn web_reader_propagates_conflict_error_from_repo() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    resource_repo.fail_once("create", DomainError::Conflict("duplicate".to_string()));
    let web_reader_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());
    let service = WebReaderService::new(resource_repo, web_reader_meta_repo, location_repo);

    let input = NewWebReaderInput {
        title: "Reader".to_string(),
        notes: None,
        url: "https://example.com/a".to_string(),
        site_name: None,
        last_checked_chapter: None,
    };

    block_on(async {
        let error = service
            .add_web_reader(input)
            .await
            .expect_err("conflict should propagate");
        assert!(matches!(error, DomainError::Conflict(_)));
    });
}

#[test]
fn web_reader_propagates_not_found_error_from_repo() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    resource_repo.fail_once("get_by_id", DomainError::NotFound("missing".to_string()));
    let web_reader_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());
    let service = WebReaderService::new(resource_repo, web_reader_meta_repo, location_repo);

    block_on(async {
        let error = service
            .web_reader_detail(Uuid::new_v4())
            .await
            .expect_err("not found should propagate");
        assert!(matches!(error, DomainError::NotFound(_)));
    });
}

#[test]
fn web_reader_propagates_internal_error_from_repo() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let resource_id = seed_web_reader_resource(&resource_repo);
    let web_reader_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    web_reader_meta_repo
        .metas
        .lock()
        .expect("web reader meta lock")
        .insert(
            resource_id,
            WebReaderMeta {
                resource_id,
                url: "https://example.com".to_string(),
                site_name: None,
                last_checked_chapter: None,
            },
        );
    let location_repo = Arc::new(MockLocationRepository::default());
    location_repo.fail_once("list", DomainError::InternalError("io".to_string()));

    let service = WebReaderService::new(resource_repo, web_reader_meta_repo, location_repo);

    block_on(async {
        let error = service
            .web_reader_detail(resource_id)
            .await
            .expect_err("internal should propagate");
        assert!(matches!(error, DomainError::InternalError(_)));
    });
}

#[test]
fn web_reader_location_happy_path_adds_location() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let web_reader_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    let location_repo = Arc::new(MockLocationRepository::default());
    let service = WebReaderService::new(resource_repo, web_reader_meta_repo, location_repo);

    let location_input = NewLocationInput {
        device_id: "dev-1".to_string(),
        path_or_url: "https://example.com".to_string(),
        storage_type: "platform".to_string(),
    };

    block_on(async {
        let location = service
            .add_web_reader_location(Uuid::new_v4(), location_input)
            .await
            .expect("add location");
        assert_eq!(location.device_id, "dev-1");
        assert_eq!(location.storage_type, StorageType::Platform);
    });
}
