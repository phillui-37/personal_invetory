use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use domain::progress::{ProgressRepository, ResourceProgress};
use domain::tag::{ResourceTagRepository, Tag, TagRepository};
use domain::{
    DomainError, EbookMeta, EbookMetaRepository, LocationRepository, NewEbookMeta, NewResource,
    NewResourceLocation, NewWebReaderMeta, Resource, ResourceLocation, ResourceRepository,
    ResourceType, StorageType, UpdateResource, WebReaderMeta, WebReaderMetaRepository,
};
use futures::executor::block_on;
use services::{
    EbookService, NewEbookInput, NewLocationInput, NewWebReaderInput, ProgressService, TagService,
    UpdateEbookInput, WebReaderService,
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
            check_interval_secs: input.check_interval_secs,
            last_checked_at: input.last_checked_at,
            progress_css_selector: input.progress_css_selector,
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
fn ebook_batch_update_reports_partial_failures_without_dropping_successes() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let first_id = seed_ebook_resource(&resource_repo);
    let second_id = seed_ebook_resource(&resource_repo);
    let missing_id = Uuid::new_v4();

    let ebook_meta_repo = Arc::new(MockEbookMetaRepository::default());
    for resource_id in [first_id, second_id] {
        ebook_meta_repo
            .metas
            .lock()
            .expect("ebook meta lock")
            .insert(
                resource_id,
                EbookMeta {
                    resource_id,
                    author: Some("Original".to_string()),
                    isbn: None,
                    publisher: None,
                    language: Some("en".to_string()),
                    file_format: Some("pdf".to_string()),
                },
            );
    }
    let location_repo = Arc::new(MockLocationRepository::default());
    let service = EbookService::new(resource_repo, ebook_meta_repo.clone(), location_repo);

    let (updated, failed) = block_on(service.batch_update_ebooks(
        vec![first_id, missing_id, second_id],
        UpdateEbookInput {
            title: Some("Retitled".to_string()),
            notes: Some("updated in bulk".to_string()),
            author: Some("Batch Author".to_string()),
            isbn: None,
            publisher: Some("Batch Publisher".to_string()),
            language: Some("ja".to_string()),
            file_format: Some("epub".to_string()),
        },
    ));

    assert_eq!(updated, 2);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].0, missing_id);
    assert!(!failed[0].1.is_empty());

    for resource_id in [first_id, second_id] {
        let meta = block_on(ebook_meta_repo.get(resource_id)).expect("updated ebook meta");
        assert_eq!(meta.author.as_deref(), Some("Batch Author"));
        assert_eq!(meta.publisher.as_deref(), Some("Batch Publisher"));
        assert_eq!(meta.language.as_deref(), Some("ja"));
        assert_eq!(meta.file_format.as_deref(), Some("epub"));
    }
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
        check_interval_secs: None,
        progress_css_selector: None,
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
        check_interval_secs: None,
        progress_css_selector: None,
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
        check_interval_secs: None,
        progress_css_selector: None,
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
                check_interval_secs: None,
                last_checked_at: None,
                progress_css_selector: None,
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

#[test]
fn web_reader_batch_copy_preserves_urls_while_copying_shared_meta_fields() {
    let resource_repo = Arc::new(MockResourceRepository::default());
    let source_id = seed_web_reader_resource(&resource_repo);
    let first_target_id = seed_web_reader_resource(&resource_repo);
    let second_target_id = seed_web_reader_resource(&resource_repo);
    let web_reader_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    {
        let mut metas = web_reader_meta_repo
            .metas
            .lock()
            .expect("web reader meta lock");
        metas.insert(
            source_id,
            WebReaderMeta {
                resource_id: source_id,
                url: "https://source.example/series".to_string(),
                site_name: Some("Source Site".to_string()),
                last_checked_chapter: Some("ch-42".to_string()),
                check_interval_secs: Some(7200),
                last_checked_at: None,
                progress_css_selector: Some(".chapter.current".to_string()),
            },
        );
        for (resource_id, url) in [
            (first_target_id, "https://target-one.example/series"),
            (second_target_id, "https://target-two.example/series"),
        ] {
            metas.insert(
                resource_id,
                WebReaderMeta {
                    resource_id,
                    url: url.to_string(),
                    site_name: Some("Old Site".to_string()),
                    last_checked_chapter: Some("old".to_string()),
                    check_interval_secs: Some(60),
                    last_checked_at: None,
                    progress_css_selector: Some(".old-selector".to_string()),
                },
            );
        }
    }
    let location_repo = Arc::new(MockLocationRepository::default());
    let service = WebReaderService::new(resource_repo, web_reader_meta_repo.clone(), location_repo);

    let (updated, failed) = block_on(
        service.batch_copy_web_reader_meta(source_id, vec![first_target_id, second_target_id]),
    );

    assert_eq!(updated, 2);
    assert!(failed.is_empty());

    let metas = web_reader_meta_repo
        .metas
        .lock()
        .expect("web reader meta lock");
    for (resource_id, expected_url) in [
        (first_target_id, "https://target-one.example/series"),
        (second_target_id, "https://target-two.example/series"),
    ] {
        let meta = metas.get(&resource_id).expect("target meta");
        assert_eq!(meta.url, expected_url);
        assert_eq!(meta.site_name.as_deref(), Some("Source Site"));
        assert_eq!(meta.last_checked_chapter.as_deref(), Some("ch-42"));
        assert_eq!(meta.check_interval_secs, Some(7200));
        assert_eq!(
            meta.progress_css_selector.as_deref(),
            Some(".chapter.current")
        );
    }
}

// ── ChapterCheckService tests ──────────────────────────────────────────────

use domain::{ChapterCheck, ChapterCheckRepository, Notification, NotificationRepository};
use plugins::{CheckResult, PluginError, WebChecker};
use services::ChapterCheckService;

struct MockWebChecker {
    result: Mutex<plugins::CheckResult>,
    error: Mutex<Option<PluginError>>,
    call_count: Mutex<usize>,
}

impl MockWebChecker {
    fn new_returning(result: CheckResult) -> Self {
        Self {
            result: Mutex::new(result),
            error: Mutex::new(None),
            call_count: Mutex::new(0),
        }
    }

    fn new_erroring(err: PluginError) -> Self {
        Self {
            result: Mutex::new(CheckResult::default()),
            error: Mutex::new(Some(err)),
            call_count: Mutex::new(0),
        }
    }

    fn call_count(&self) -> usize {
        *self.call_count.lock().expect("call count lock")
    }
}

impl WebChecker for MockWebChecker {
    fn check(&self, _url: &str, _known: Option<&str>) -> Result<CheckResult, PluginError> {
        *self.call_count.lock().expect("call count lock") += 1;
        if let Some(err) = self.error.lock().expect("error lock").take() {
            return Err(err);
        }
        Ok(self.result.lock().expect("result lock").clone())
    }
}

struct BlockingRuntimeWebChecker;

impl WebChecker for BlockingRuntimeWebChecker {
    fn check(&self, _url: &str, _known: Option<&str>) -> Result<CheckResult, PluginError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|err| PluginError::IoError(format!("tokio runtime: {err}")))?;
        runtime.block_on(async {
            Ok(CheckResult {
                has_new: false,
                latest_chapter: Some("ch-1".to_string()),
            })
        })
    }
}

#[derive(Default)]
struct MockChapterCheckRepository {
    checks: Mutex<Vec<ChapterCheck>>,
}

#[async_trait]
impl ChapterCheckRepository for MockChapterCheckRepository {
    async fn create(
        &self,
        resource_id: Uuid,
        has_new_chapter: bool,
        latest_chapter: Option<String>,
        error_message: Option<String>,
    ) -> Result<ChapterCheck, DomainError> {
        let check = ChapterCheck {
            id: Uuid::new_v4(),
            resource_id,
            has_new_chapter,
            latest_chapter,
            checked_at: Utc::now(),
            error_message,
        };
        self.checks.lock().expect("checks lock").push(check.clone());
        Ok(check)
    }

    async fn list(&self, resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError> {
        Ok(self
            .checks
            .lock()
            .expect("checks lock")
            .iter()
            .filter(|c| c.resource_id == resource_id)
            .cloned()
            .collect())
    }
}

#[derive(Default)]
struct MockNotificationRepository {
    notifications: Mutex<Vec<Notification>>,
}

impl MockNotificationRepository {
    fn count(&self) -> usize {
        self.notifications.lock().expect("notif lock").len()
    }
}

#[async_trait]
impl NotificationRepository for MockNotificationRepository {
    async fn create(
        &self,
        resource_id: Uuid,
        message: String,
    ) -> Result<Notification, DomainError> {
        let n = Notification {
            id: Uuid::new_v4(),
            resource_id,
            message,
            created_at: Utc::now(),
            read: false,
        };
        self.notifications
            .lock()
            .expect("notif lock")
            .push(n.clone());
        Ok(n)
    }

    async fn list(&self, unread_only: bool) -> Result<Vec<Notification>, DomainError> {
        Ok(self
            .notifications
            .lock()
            .expect("notif lock")
            .iter()
            .filter(|n| !unread_only || !n.read)
            .cloned()
            .collect())
    }

    async fn mark_read(&self, id: Uuid) -> Result<(), DomainError> {
        let mut guard = self.notifications.lock().expect("notif lock");
        let n = guard
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| DomainError::NotFound(format!("notification {id} not found")))?;
        n.read = true;
        Ok(())
    }
}

fn seed_web_meta(repo: &MockWebReaderMetaRepository, resource_id: Uuid) {
    repo.metas.lock().expect("meta lock").insert(
        resource_id,
        WebReaderMeta {
            resource_id,
            url: "https://example.com/comic".to_string(),
            site_name: Some("Example".to_string()),
            last_checked_chapter: Some("ch-1".to_string()),
            check_interval_secs: None,
            last_checked_at: None,
            progress_css_selector: None,
        },
    );
}

fn build_chapter_check_service(
    checker: Arc<MockWebChecker>,
    web_meta_repo: Arc<MockWebReaderMetaRepository>,
) -> (
    ChapterCheckService<
        MockWebChecker,
        MockChapterCheckRepository,
        MockNotificationRepository,
        MockWebReaderMetaRepository,
    >,
    Arc<MockChapterCheckRepository>,
    Arc<MockNotificationRepository>,
) {
    let check_repo = Arc::new(MockChapterCheckRepository::default());
    let notif_repo = Arc::new(MockNotificationRepository::default());
    let service = ChapterCheckService::new(
        checker,
        check_repo.clone(),
        notif_repo.clone(),
        web_meta_repo,
        None,
    );
    (service, check_repo, notif_repo)
}

#[tokio::test]
async fn chapter_check_service_new_chapter_creates_notification() {
    let resource_id = Uuid::new_v4();
    let web_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    seed_web_meta(&web_meta_repo, resource_id);

    let checker = Arc::new(MockWebChecker::new_returning(CheckResult {
        has_new: true,
        latest_chapter: Some("ch-2".to_string()),
    }));
    let (service, check_repo, notif_repo) =
        build_chapter_check_service(checker.clone(), web_meta_repo.clone());

    let check = service
        .check_resource(resource_id)
        .await
        .expect("check resource");

    assert!(check.has_new_chapter);
    assert_eq!(check.latest_chapter.as_deref(), Some("ch-2"));
    assert_eq!(checker.call_count(), 1);
    assert_eq!(notif_repo.count(), 1);
    assert_eq!(check_repo.list(resource_id).await.expect("list").len(), 1);

    // web meta updated with new chapter
    let meta = web_meta_repo.get(resource_id).await.expect("get meta");
    assert_eq!(meta.last_checked_chapter.as_deref(), Some("ch-2"));
    assert!(meta.last_checked_at.is_some());
}

#[tokio::test]
async fn chapter_check_service_no_new_chapter_skips_notification() {
    let resource_id = Uuid::new_v4();
    let web_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    seed_web_meta(&web_meta_repo, resource_id);

    let checker = Arc::new(MockWebChecker::new_returning(CheckResult {
        has_new: false,
        latest_chapter: Some("ch-1".to_string()),
    }));
    let (service, _, notif_repo) = build_chapter_check_service(checker, web_meta_repo.clone());

    let check = service
        .check_resource(resource_id)
        .await
        .expect("check resource");

    assert!(!check.has_new_chapter);
    assert_eq!(notif_repo.count(), 0);

    let meta = web_meta_repo.get(resource_id).await.expect("get meta");
    assert!(
        meta.last_checked_at.is_some(),
        "successful checks should record last_checked_at even when nothing changed"
    );
}

#[tokio::test]
async fn chapter_check_service_io_error_records_error_check() {
    let resource_id = Uuid::new_v4();
    let web_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    seed_web_meta(&web_meta_repo, resource_id);

    let checker = Arc::new(MockWebChecker::new_erroring(PluginError::IoError(
        "connection timeout".to_string(),
    )));
    let (service, check_repo, notif_repo) = build_chapter_check_service(checker, web_meta_repo);

    let check = service
        .check_resource(resource_id)
        .await
        .expect("error check should still succeed with error recorded");

    assert!(!check.has_new_chapter);
    assert_eq!(check.error_message.as_deref(), Some("connection timeout"));
    assert_eq!(notif_repo.count(), 0);
    assert_eq!(check_repo.list(resource_id).await.expect("list").len(), 1);
}

#[tokio::test]
async fn chapter_check_service_unknown_resource_returns_not_found() {
    let web_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    let checker = Arc::new(MockWebChecker::new_returning(CheckResult::default()));
    let (service, _, _) = build_chapter_check_service(checker, web_meta_repo);

    let err = service
        .check_resource(Uuid::new_v4())
        .await
        .expect_err("unknown resource should return error");
    assert!(matches!(err, DomainError::NotFound(_)));
}

#[tokio::test]
async fn chapter_check_service_runs_checker_without_nested_runtime_panic() {
    let resource_id = Uuid::new_v4();
    let web_meta_repo = Arc::new(MockWebReaderMetaRepository::default());
    seed_web_meta(&web_meta_repo, resource_id);

    let service = ChapterCheckService::new(
        Arc::new(BlockingRuntimeWebChecker),
        Arc::new(MockChapterCheckRepository::default()),
        Arc::new(MockNotificationRepository::default()),
        web_meta_repo,
        None,
    );

    let check = service
        .check_resource(resource_id)
        .await
        .expect("checker should run without panicking inside async runtime");

    assert!(!check.has_new_chapter);
    assert_eq!(check.latest_chapter.as_deref(), Some("ch-1"));
}

// ── DeviceService ──────────────────────────────────────────────

use domain::device::{Device, DeviceRepository};

struct FakeDeviceRepo {
    devices: std::sync::Mutex<Vec<Device>>,
}

impl FakeDeviceRepo {
    fn empty() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            devices: std::sync::Mutex::new(vec![]),
        })
    }
}

#[async_trait::async_trait]
impl DeviceRepository for FakeDeviceRepo {
    async fn all_with_counts(&self) -> Result<Vec<Device>, domain::DomainError> {
        Ok(self.devices.lock().unwrap().clone())
    }
    async fn active_by_device_id(
        &self,
        device_id: &str,
    ) -> Result<Option<Device>, domain::DomainError> {
        let found = self
            .devices
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.device_id == device_id && d.delinked_at.is_none())
            .cloned();
        Ok(found)
    }
    async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<Device, domain::DomainError> {
        let d = Device {
            id: uuid::Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            device_name: device_name.map(str::to_string),
            linked_at: chrono::Utc::now(),
            delinked_at: None,
            location_count: 0,
        };
        self.devices.lock().unwrap().push(d.clone());
        Ok(d)
    }
    async fn delink(
        &self,
        device_id: &str,
        at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), domain::DomainError> {
        let mut devices = self.devices.lock().unwrap();
        let total = devices.iter().filter(|d| d.device_id == device_id).count();
        if total == 0 {
            return Err(domain::DomainError::NotFound(format!(
                "{device_id} not found"
            )));
        }
        let active = devices
            .iter_mut()
            .find(|d| d.device_id == device_id && d.delinked_at.is_none());
        match active {
            None => Err(domain::DomainError::Conflict(format!(
                "{device_id} already delinked"
            ))),
            Some(d) => {
                d.delinked_at = Some(at);
                Ok(())
            }
        }
    }
}

#[tokio::test]
async fn device_service_list_enriches_with_is_current() {
    let repo = FakeDeviceRepo::empty();
    repo.register("current-dev", None).await.unwrap();
    repo.register("other-dev", Some("Other")).await.unwrap();
    let svc = services::DeviceService::new(repo, "current-dev".into(), "myhostname".into());
    let list = svc.list().await.expect("list");
    let current = list.iter().find(|d| d.device_id == "current-dev").unwrap();
    let other = list.iter().find(|d| d.device_id == "other-dev").unwrap();
    assert!(current.is_current);
    assert!(!other.is_current);
    // hostname fallback for current device when device_name is None
    assert_eq!(current.device_name, "myhostname");
    // device_id fallback for other devices when device_name is None
    assert_eq!(other.device_name, "Other");
}

#[tokio::test]
async fn device_service_current_returns_not_found_when_unregistered() {
    let repo = FakeDeviceRepo::empty();
    let svc = services::DeviceService::new(repo, "missing".into(), "h".into());
    let err = svc.current().await.expect_err("should be NotFound");
    assert!(matches!(err, domain::DomainError::NotFound(_)));
}

#[tokio::test]
async fn device_service_register_returns_device_info() {
    let repo = FakeDeviceRepo::empty();
    let svc = services::DeviceService::new(repo, "dev1".into(), "hostname".into());
    let info = svc.register("dev1", Some("My PC")).await.expect("register");
    assert_eq!(info.device_id, "dev1");
    assert_eq!(info.device_name, "My PC");
    assert!(info.is_current);
}

#[tokio::test]
async fn device_service_register_rejects_empty_device_id() {
    let repo = FakeDeviceRepo::empty();
    let svc = services::DeviceService::new(repo, "dev1".into(), "h".into());
    let err = svc
        .register("", None)
        .await
        .expect_err("empty device_id must fail");
    assert!(matches!(err, domain::DomainError::ValidationError(_)));

    // whitespace-only should also fail
    let err2 = svc
        .register("   ", None)
        .await
        .expect_err("whitespace device_id must fail");
    assert!(matches!(err2, domain::DomainError::ValidationError(_)));
}

#[tokio::test]
async fn device_service_delink_self_returns_validation_error() {
    let repo = FakeDeviceRepo::empty();
    repo.register("dev1", None).await.unwrap();
    let svc = services::DeviceService::new(repo, "dev1".into(), "h".into());
    let err = svc.delink("dev1").await.expect_err("cannot delink self");
    assert!(matches!(err, domain::DomainError::ValidationError(_)));
}

#[tokio::test]
async fn device_service_delink_other_device_succeeds() {
    let repo = FakeDeviceRepo::empty();
    repo.register("dev1", None).await.unwrap();
    repo.register("dev2", None).await.unwrap();
    let svc = services::DeviceService::new(repo, "dev1".into(), "h".into());
    svc.delink("dev2").await.expect("delink other device");
}

// ── ProgressService mock & tests ─────────────────────────────────────────────

#[derive(Default)]
struct MockProgressRepository {
    storage: Mutex<HashMap<String, ResourceProgress>>,
    upsert_calls: Mutex<usize>,
}

#[async_trait]
impl ProgressRepository for MockProgressRepository {
    async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError> {
        Ok(self
            .storage
            .lock()
            .expect("progress storage lock")
            .get(resource_id)
            .cloned())
    }

    async fn upsert(
        &self,
        resource_id: &str,
        progress: f64,
        notes: Option<&str>,
    ) -> Result<ResourceProgress, DomainError> {
        *self.upsert_calls.lock().expect("upsert calls lock") += 1;
        let record = ResourceProgress {
            resource_id: resource_id.to_string(),
            progress,
            notes: notes.map(|s| s.to_string()),
            updated_at: Utc::now(),
        };
        self.storage
            .lock()
            .expect("progress storage lock")
            .insert(resource_id.to_string(), record.clone());
        Ok(record)
    }
}

#[tokio::test]
async fn progress_service_get_returns_none_when_not_set() {
    let repo = Arc::new(MockProgressRepository::default());
    let svc = ProgressService::new(repo);
    let result = svc.get("resource-1").await.expect("get must not error");
    assert!(result.is_none());
}

#[tokio::test]
async fn progress_service_upsert_validates_range() {
    let repo = Arc::new(MockProgressRepository::default());
    let svc = ProgressService::new(Arc::clone(&repo) as Arc<dyn ProgressRepository>);

    let err_high = svc
        .upsert("resource-1", 1.5, None)
        .await
        .expect_err("progress > 1.0 must fail");
    assert!(matches!(err_high, DomainError::ValidationError(_)));

    let err_low = svc
        .upsert("resource-1", -0.1, None)
        .await
        .expect_err("progress < 0.0 must fail");
    assert!(matches!(err_low, DomainError::ValidationError(_)));

    let success_min = svc
        .upsert("resource-1", 0.0, None)
        .await
        .expect("progress = 0.0 must succeed");
    assert_eq!(success_min.progress, 0.0);

    let success_max = svc
        .upsert("resource-1", 1.0, None)
        .await
        .expect("progress = 1.0 must succeed");
    assert_eq!(success_max.progress, 1.0);

    let calls = *repo.upsert_calls.lock().expect("upsert calls lock");
    assert_eq!(
        calls, 2,
        "repo.upsert must be called twice for both valid boundary values"
    );
}

#[tokio::test]
async fn progress_service_upsert_round_trips() {
    let repo: Arc<dyn ProgressRepository> = Arc::new(MockProgressRepository::default());
    let svc = ProgressService::new(Arc::clone(&repo));

    let created = svc
        .upsert("resource-1", 0.25, Some("start"))
        .await
        .expect("upsert must succeed");
    assert_eq!(created.resource_id, "resource-1");
    assert_eq!(created.progress, 0.25);
    assert_eq!(created.notes, Some("start".to_string()));

    let fetched = svc
        .get("resource-1")
        .await
        .expect("get must not error")
        .expect("must have value after upsert");
    assert_eq!(fetched.progress, 0.25);
    assert_eq!(fetched.notes, Some("start".to_string()));

    svc.upsert("resource-1", 0.75, None)
        .await
        .expect("second upsert must succeed");

    let updated = svc
        .get("resource-1")
        .await
        .expect("get must not error")
        .expect("must have value after second upsert");
    assert_eq!(updated.progress, 0.75);
    assert!(updated.notes.is_none());
}

// ── TagService mocks & tests ──────────────────────────────────────────────────

#[derive(Default)]
struct MockTagRepository {
    tags: Mutex<HashMap<String, Tag>>,
    create_calls: Mutex<usize>,
    created_names: Mutex<Vec<String>>,
}

#[async_trait]
impl TagRepository for MockTagRepository {
    async fn list(&self) -> Result<Vec<Tag>, DomainError> {
        Ok(self.tags.lock().unwrap().values().cloned().collect())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Tag>, DomainError> {
        Ok(self
            .tags
            .lock()
            .unwrap()
            .values()
            .find(|t| t.id == id)
            .cloned())
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Tag>, DomainError> {
        Ok(self
            .tags
            .lock()
            .unwrap()
            .values()
            .find(|t| t.name == name)
            .cloned())
    }

    async fn create(&self, name: &str) -> Result<Tag, DomainError> {
        *self.create_calls.lock().unwrap() += 1;
        self.created_names.lock().unwrap().push(name.to_string());
        let tag = Tag {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            created_at: Utc::now(),
        };
        self.tags
            .lock()
            .unwrap()
            .insert(tag.id.clone(), tag.clone());
        Ok(tag)
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        self.tags
            .lock()
            .unwrap()
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| DomainError::NotFound(id.to_string()))
    }
}

#[derive(Default)]
struct MockResourceTagRepository {
    associations: Mutex<HashSet<(String, String)>>,
    tag_store: Arc<MockTagRepository>,
}

impl MockResourceTagRepository {
    fn with_tags(tag_store: Arc<MockTagRepository>) -> Self {
        Self {
            associations: Mutex::default(),
            tag_store,
        }
    }
}

#[async_trait]
impl ResourceTagRepository for MockResourceTagRepository {
    async fn tags_for_resource(&self, resource_id: &str) -> Result<Vec<Tag>, DomainError> {
        let assocs = self.associations.lock().unwrap();
        let tag_ids: Vec<String> = assocs
            .iter()
            .filter(|(rid, _)| rid == resource_id)
            .map(|(_, tid)| tid.clone())
            .collect();
        let tags_map = self.tag_store.tags.lock().unwrap();
        Ok(tag_ids
            .iter()
            .filter_map(|id| tags_map.get(id).cloned())
            .collect())
    }

    async fn attach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        self.associations
            .lock()
            .unwrap()
            .insert((resource_id.to_string(), tag_id.to_string()));
        Ok(())
    }

    async fn detach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        let removed = self
            .associations
            .lock()
            .unwrap()
            .remove(&(resource_id.to_string(), tag_id.to_string()));
        if removed {
            Ok(())
        } else {
            Err(DomainError::NotFound(format!(
                "association ({resource_id}, {tag_id}) not found"
            )))
        }
    }

    async fn resource_ids_with_tag_id(&self, tag_id: &str) -> Result<Vec<String>, DomainError> {
        Ok(self
            .associations
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, tid)| tid == tag_id)
            .map(|(rid, _)| rid.clone())
            .collect())
    }
}

fn make_tag_service() -> (
    Arc<MockTagRepository>,
    Arc<MockResourceTagRepository>,
    TagService,
) {
    let tag_repo = Arc::new(MockTagRepository::default());
    let resource_tag_repo = Arc::new(MockResourceTagRepository::with_tags(Arc::clone(&tag_repo)));
    let svc = TagService::new(
        Arc::clone(&tag_repo) as Arc<dyn TagRepository>,
        Arc::clone(&resource_tag_repo) as Arc<dyn ResourceTagRepository>,
    );
    (tag_repo, resource_tag_repo, svc)
}

#[tokio::test]
async fn tag_service_create_normalises_name() {
    let (tag_repo, _, svc) = make_tag_service();

    let tag = svc.create("  Sci-Fi  ").await.expect("create must succeed");
    assert_eq!(tag.name, "sci-fi");

    let created_names = tag_repo.created_names.lock().unwrap();
    assert_eq!(created_names.as_slice(), &["sci-fi"]);
}

#[tokio::test]
async fn tag_service_create_rejects_empty_name() {
    let (tag_repo, _, svc) = make_tag_service();

    let err = svc
        .create("   ")
        .await
        .expect_err("whitespace-only must fail");
    assert!(matches!(err, DomainError::ValidationError(_)));

    let calls = *tag_repo.create_calls.lock().unwrap();
    assert_eq!(
        calls, 0,
        "repo.create must NOT be called on validation failure"
    );
}

#[tokio::test]
async fn tag_service_attach_tag_to_resource() {
    let (tag_repo, resource_tag_repo, svc) = make_tag_service();

    let tag = tag_repo.create("action").await.expect("seed tag");
    svc.attach_tag("res-1", &tag.id)
        .await
        .expect("attach must succeed");

    let assocs = resource_tag_repo.associations.lock().unwrap();
    assert!(assocs.contains(&("res-1".to_string(), tag.id.clone())));
}

#[tokio::test]
async fn tag_service_detach_tag_from_resource() {
    let (tag_repo, resource_tag_repo, svc) = make_tag_service();

    let tag = tag_repo.create("action").await.expect("seed tag");
    resource_tag_repo
        .associations
        .lock()
        .unwrap()
        .insert(("res-1".to_string(), tag.id.clone()));

    svc.detach_tag("res-1", &tag.id)
        .await
        .expect("detach must succeed");

    let assocs = resource_tag_repo.associations.lock().unwrap();
    assert!(!assocs.contains(&("res-1".to_string(), tag.id.clone())));
}

#[tokio::test]
async fn tag_service_tags_for_resource() {
    let (tag_repo, resource_tag_repo, svc) = make_tag_service();

    let t1 = tag_repo.create("sci-fi").await.expect("seed t1");
    let t2 = tag_repo.create("action").await.expect("seed t2");
    {
        let mut assocs = resource_tag_repo.associations.lock().unwrap();
        assocs.insert(("res-1".to_string(), t1.id.clone()));
        assocs.insert(("res-1".to_string(), t2.id.clone()));
    }

    let mut tags = svc
        .tags_for_resource("res-1")
        .await
        .expect("tags_for_resource must succeed");
    tags.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].name, "action");
    assert_eq!(tags[1].name, "sci-fi");
}

#[tokio::test]
async fn tag_service_filter_resource_ids_by_tag() {
    let (tag_repo, resource_tag_repo, svc) = make_tag_service();

    // Seed via service so normalization is exercised end-to-end
    let tag = svc.create("  Sci-Fi  ").await.expect("create must succeed");
    assert_eq!(tag.name, "sci-fi");

    {
        let mut assocs = resource_tag_repo.associations.lock().unwrap();
        assocs.insert(("res-a".to_string(), tag.id.clone()));
        assocs.insert(("res-b".to_string(), tag.id.clone()));
    }

    let mut ids = svc
        .resource_ids_for_tag_name(" sci-fi ")
        .await
        .expect("lookup must succeed");
    ids.sort();
    assert_eq!(ids, vec!["res-a".to_string(), "res-b".to_string()]);

    // Unknown tag → empty vec, not an error
    let empty = svc
        .resource_ids_for_tag_name("unknown")
        .await
        .expect("unknown tag must return empty vec");
    assert!(empty.is_empty());

    // Whitespace-only → ValidationError
    let err = svc
        .resource_ids_for_tag_name("   ")
        .await
        .expect_err("empty name must fail");
    assert!(matches!(err, DomainError::ValidationError(_)));
}

#[tokio::test]
async fn tag_service_list_returns_all_tags() {
    let (tag_repo, _, svc) = make_tag_service();

    let t1 = tag_repo.create("action").await.expect("seed t1");
    let t2 = tag_repo.create("sci-fi").await.expect("seed t2");

    let mut tags = svc.list().await.expect("list must succeed");
    tags.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].id, t1.id);
    assert_eq!(tags[0].name, "action");
    assert_eq!(tags[1].id, t2.id);
    assert_eq!(tags[1].name, "sci-fi");
}

#[tokio::test]
async fn tag_service_delete_propagates_not_found() {
    let (tag_repo, _, svc) = make_tag_service();

    let err = svc
        .delete("missing-tag")
        .await
        .expect_err("delete of missing tag must fail");
    assert!(matches!(err, DomainError::NotFound(_)));

    let tag = tag_repo.create("action").await.expect("create tag");
    svc.delete(&tag.id)
        .await
        .expect("delete of existing tag must succeed");

    let fetched = tag_repo
        .get_by_id(&tag.id)
        .await
        .expect("get_by_id must succeed");
    assert!(fetched.is_none(), "tag must be deleted");
}
