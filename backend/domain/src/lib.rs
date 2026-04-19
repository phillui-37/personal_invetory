pub mod dedup;
pub mod device;
pub mod ecosystem;
pub mod sync;
pub mod vault;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ResourceType {
    Ebook,
    WebReader,
    Image,
    Video,
    Game,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum StorageType {
    LocalFs,
    Nas,
    Platform,
    Portable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Resource {
    pub id: Uuid,
    pub title: String,
    pub notes: Option<String>,
    pub resource_type: ResourceType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct EbookMeta {
    pub resource_id: Uuid,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct WebReaderMeta {
    pub resource_id: Uuid,
    pub url: String,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
    pub check_interval_secs: Option<u64>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub progress_css_selector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ChapterCheck {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub has_new_chapter: bool,
    pub latest_chapter: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Notification {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ResourceLocation {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: StorageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewResource {
    pub title: String,
    pub notes: Option<String>,
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateResource {
    pub title: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewEbookMeta {
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewWebReaderMeta {
    pub url: String,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
    pub check_interval_secs: Option<u64>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub progress_css_selector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewResourceLocation {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: StorageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ImageMeta {
    pub resource_id: Uuid,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_format: Option<String>,
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct VideoMeta {
    pub resource_id: Uuid,
    pub duration_secs: Option<u64>,
    pub file_format: Option<String>,
    pub resolution: Option<String>,
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct GameMeta {
    pub resource_id: Uuid,
    pub platform: Option<String>,
    pub store: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub manual_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewImageMeta {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_format: Option<String>,
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewVideoMeta {
    pub duration_secs: Option<u64>,
    pub file_format: Option<String>,
    pub resolution: Option<String>,
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewGameMeta {
    pub platform: Option<String>,
    pub store: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub manual_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    NotFound(String),
    ValidationError(String),
    Conflict(String),
    InternalError(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SearchStrategyKind {
    #[default]
    Like,
    Fuzzy,
}

impl SearchStrategyKind {
    pub fn from_config(value: Option<&str>) -> Self {
        let Some(value) = value else {
            return Self::Like;
        };

        if value.eq_ignore_ascii_case("fuzzy") {
            Self::Fuzzy
        } else {
            Self::Like
        }
    }
}

#[async_trait]
pub trait ResourceRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Resource>, DomainError>;
    async fn search(&self, query: &str) -> Result<Vec<Resource>, DomainError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError>;
    async fn create(&self, input: NewResource) -> Result<Resource, DomainError>;
    async fn update(&self, id: Uuid, input: UpdateResource) -> Result<Resource, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
pub trait EbookMetaRepository: Send + Sync {
    async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError>;
    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewEbookMeta,
    ) -> Result<EbookMeta, DomainError>;
}

#[async_trait]
pub trait WebReaderMetaRepository: Send + Sync {
    async fn get(&self, resource_id: Uuid) -> Result<WebReaderMeta, DomainError>;
    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewWebReaderMeta,
    ) -> Result<WebReaderMeta, DomainError>;
}

#[async_trait]
pub trait ImageMetaRepository: Send + Sync {
    async fn get(&self, resource_id: Uuid) -> Result<ImageMeta, DomainError>;
    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewImageMeta,
    ) -> Result<ImageMeta, DomainError>;
}

#[async_trait]
pub trait VideoMetaRepository: Send + Sync {
    async fn get(&self, resource_id: Uuid) -> Result<VideoMeta, DomainError>;
    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewVideoMeta,
    ) -> Result<VideoMeta, DomainError>;
}

#[async_trait]
pub trait GameMetaRepository: Send + Sync {
    async fn get(&self, resource_id: Uuid) -> Result<GameMeta, DomainError>;
    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewGameMeta,
    ) -> Result<GameMeta, DomainError>;
}

#[async_trait]
pub trait LocationRepository: Send + Sync {
    async fn list(&self, resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError>;
    async fn add(
        &self,
        resource_id: Uuid,
        input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError>;
    async fn remove(&self, resource_id: Uuid, location_id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
pub trait ChapterCheckRepository: Send + Sync {
    async fn create(
        &self,
        resource_id: Uuid,
        has_new_chapter: bool,
        latest_chapter: Option<String>,
        error_message: Option<String>,
    ) -> Result<ChapterCheck, DomainError>;
    async fn list(&self, resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError>;
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(
        &self,
        resource_id: Uuid,
        message: String,
    ) -> Result<Notification, DomainError>;
    async fn list(&self, unread_only: bool) -> Result<Vec<Notification>, DomainError>;
    async fn mark_read(&self, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
pub trait PushNotifier: Send + Sync {
    async fn send(
        &self,
        resource_id: Uuid,
        title: &str,
        body: &str,
    ) -> Result<(), DomainError>;
}

/// Synchronous broadcaster for real-time notification delivery (SSE, etc).
pub trait NotificationBroadcaster: Send + Sync {
    fn broadcast(&self, notification: &Notification);
}

#[async_trait]
pub trait SearchStrategy: Send + Sync {
    fn kind(&self) -> SearchStrategyKind;
    async fn search(
        &self,
        repository: &dyn ResourceRepository,
        query: &str,
    ) -> Result<Vec<Resource>, DomainError>;
}

pub fn domain_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{
        GameMeta, ImageMeta, NewGameMeta, NewImageMeta, NewVideoMeta, Resource, ResourceType,
        StorageType, VideoMeta,
    };
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn resource_holds_core_fields() {
        let now = Utc::now();
        let resource = Resource {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid UUID"),
            title: "Book".to_string(),
            notes: Some("note".to_string()),
            resource_type: ResourceType::Ebook,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(resource.title, "Book");
        assert!(matches!(resource.resource_type, ResourceType::Ebook));
    }

    #[test]
    fn storage_type_supports_portable() {
        let storage = StorageType::Portable;
        assert!(matches!(storage, StorageType::Portable));
    }

    #[test]
    fn serde_round_trip_for_resource_type() {
        let serialized =
            serde_json::to_string(&ResourceType::WebReader).expect("serialize resource type");
        let deserialized: ResourceType =
            serde_json::from_str(&serialized).expect("deserialize resource type");
        assert_eq!(deserialized, ResourceType::WebReader);
    }

    #[test]
    fn serde_deserializes_phase_three_resource_types() {
        for raw in ["\"Image\"", "\"Video\"", "\"Game\""] {
            let parsed = serde_json::from_str::<ResourceType>(raw);
            assert!(parsed.is_ok(), "expected {raw} to deserialize into ResourceType");
        }
    }

    #[test]
    fn image_meta_holds_expected_fields() {
        let meta = ImageMeta {
            resource_id: Uuid::new_v4(),
            width: Some(1920),
            height: Some(1080),
            file_format: Some("png".to_string()),
            file_size_bytes: Some(204800),
        };
        assert_eq!(meta.width, Some(1920));
        assert_eq!(meta.file_format.as_deref(), Some("png"));
    }

    #[test]
    fn video_meta_holds_expected_fields() {
        let meta = VideoMeta {
            resource_id: Uuid::new_v4(),
            duration_secs: Some(3600),
            file_format: Some("mkv".to_string()),
            resolution: Some("1920x1080".to_string()),
            file_size_bytes: Some(1_000_000),
        };
        assert_eq!(meta.duration_secs, Some(3600));
        assert_eq!(meta.file_format.as_deref(), Some("mkv"));
    }

    #[test]
    fn game_meta_holds_expected_fields() {
        let meta = GameMeta {
            resource_id: Uuid::new_v4(),
            platform: Some("Nintendo Switch".to_string()),
            store: Some("eShop".to_string()),
            developer: Option::<String>::None,
            publisher: Option::<String>::None,
            manual_notes: Some("physical cartridge".to_string()),
        };
        assert_eq!(meta.platform.as_deref(), Some("Nintendo Switch"));
        assert_eq!(meta.manual_notes.as_deref(), Some("physical cartridge"));
    }

    #[test]
    fn new_image_meta_can_be_constructed() {
        let input = NewImageMeta {
            width: Some(800),
            height: Some(600),
            file_format: Some("jpg".to_string()),
            file_size_bytes: None,
        };
        assert_eq!(input.width, Some(800));
    }

    #[test]
    fn new_video_meta_can_be_constructed() {
        let input = NewVideoMeta {
            duration_secs: Some(120),
            file_format: Some("mp4".to_string()),
            resolution: None,
            file_size_bytes: None,
        };
        assert_eq!(input.duration_secs, Some(120));
    }

    #[test]
    fn new_game_meta_can_be_constructed() {
        let input = NewGameMeta {
            platform: Some("Steam".to_string()),
            store: Some("Steam".to_string()),
            developer: None,
            publisher: None,
            manual_notes: None,
        };
        assert_eq!(input.platform.as_deref(), Some("Steam"));
    }
}

#[cfg(test)]
mod repository_contract_tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use futures::executor::block_on;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    struct InMemoryResourceRepository {
        items: Mutex<HashMap<Uuid, Resource>>,
    }

    #[async_trait]
    impl ResourceRepository for InMemoryResourceRepository {
        async fn list(&self) -> Result<Vec<Resource>, DomainError> {
            let mut values: Vec<Resource> = self
                .items
                .lock()
                .expect("resource repo lock")
                .values()
                .cloned()
                .collect();
            values.sort_by(|a, b| a.title.cmp(&b.title));
            Ok(values)
        }

        async fn search(&self, query: &str) -> Result<Vec<Resource>, DomainError> {
            if query.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "query cannot be empty".to_string(),
                ));
            }

            let query_lower = query.to_lowercase();
            let mut values: Vec<Resource> = self
                .items
                .lock()
                .expect("resource repo lock")
                .values()
                .filter(|resource| resource.title.to_lowercase().contains(&query_lower))
                .cloned()
                .collect();
            values.sort_by(|a, b| a.title.cmp(&b.title));
            Ok(values)
        }

        async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
            self.items
                .lock()
                .expect("resource repo lock")
                .get(&id)
                .cloned()
                .ok_or_else(|| DomainError::NotFound(format!("resource {id} not found")))
        }

        async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
            if self
                .items
                .lock()
                .expect("resource repo lock")
                .values()
                .any(|resource| resource.title == input.title)
            {
                return Err(DomainError::Conflict(
                    "resource title already exists".to_string(),
                ));
            }

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
            self.items
                .lock()
                .expect("resource repo lock")
                .insert(id, resource.clone());
            Ok(resource)
        }

        async fn update(&self, id: Uuid, input: UpdateResource) -> Result<Resource, DomainError> {
            let mut guard = self.items.lock().expect("resource repo lock");
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
            let mut guard = self.items.lock().expect("resource repo lock");
            if guard.remove(&id).is_none() {
                return Err(DomainError::NotFound(format!("resource {id} not found")));
            }
            Ok(())
        }
    }

    struct InMemoryEbookMetaRepository {
        items: Mutex<HashMap<Uuid, EbookMeta>>,
    }

    #[async_trait]
    impl EbookMetaRepository for InMemoryEbookMetaRepository {
        async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError> {
            self.items
                .lock()
                .expect("ebook repo lock")
                .get(&resource_id)
                .cloned()
                .ok_or_else(|| DomainError::NotFound(format!("ebook meta {resource_id} not found")))
        }

        async fn upsert(
            &self,
            resource_id: Uuid,
            input: NewEbookMeta,
        ) -> Result<EbookMeta, DomainError> {
            let mut guard = self.items.lock().expect("ebook repo lock");
            let value = EbookMeta {
                resource_id,
                author: input.author,
                isbn: input.isbn,
                publisher: input.publisher,
                language: input.language,
                file_format: input.file_format,
            };
            guard.insert(resource_id, value.clone());
            Ok(value)
        }
    }

    struct InMemoryWebReaderMetaRepository {
        items: Mutex<HashMap<Uuid, WebReaderMeta>>,
    }

    #[async_trait]
    impl WebReaderMetaRepository for InMemoryWebReaderMetaRepository {
        async fn get(&self, resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
            self.items
                .lock()
                .expect("web reader repo lock")
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
            let mut guard = self.items.lock().expect("web reader repo lock");
            let value = WebReaderMeta {
                resource_id,
                url: input.url,
                site_name: input.site_name,
                last_checked_chapter: input.last_checked_chapter,
                check_interval_secs: input.check_interval_secs,
                last_checked_at: input.last_checked_at,
                progress_css_selector: input.progress_css_selector,
            };
            guard.insert(resource_id, value.clone());
            Ok(value)
        }
    }

    struct InMemoryLocationRepository {
        items: Mutex<HashMap<Uuid, Vec<ResourceLocation>>>,
    }

    #[async_trait]
    impl LocationRepository for InMemoryLocationRepository {
        async fn list(&self, resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
            Ok(self
                .items
                .lock()
                .expect("location repo lock")
                .get(&resource_id)
                .cloned()
                .unwrap_or_default())
        }

        async fn add(
            &self,
            resource_id: Uuid,
            input: NewResourceLocation,
        ) -> Result<ResourceLocation, DomainError> {
            let location = ResourceLocation {
                id: Uuid::new_v4(),
                resource_id,
                device_id: input.device_id,
                path_or_url: input.path_or_url,
                storage_type: input.storage_type,
            };
            let mut guard = self.items.lock().expect("location repo lock");
            guard.entry(resource_id).or_default().push(location.clone());
            Ok(location)
        }

        async fn remove(&self, resource_id: Uuid, location_id: Uuid) -> Result<(), DomainError> {
            let mut guard = self.items.lock().expect("location repo lock");
            let locations = guard.get_mut(&resource_id).ok_or_else(|| {
                DomainError::NotFound(format!("resource {resource_id} has no locations"))
            })?;
            let original_len = locations.len();
            locations.retain(|location| location.id != location_id);
            if locations.len() == original_len {
                return Err(DomainError::NotFound(format!(
                    "location {location_id} not found for resource {resource_id}"
                )));
            }
            Ok(())
        }
    }

    fn new_resource(title: &str, resource_type: ResourceType) -> NewResource {
        NewResource {
            title: title.to_string(),
            notes: None,
            resource_type,
        }
    }

    #[test]
    fn resource_repository_crud_and_error_contract() {
        let repository = InMemoryResourceRepository {
            items: Mutex::new(HashMap::new()),
        };

        block_on(async {
            let created = repository
                .create(new_resource("Rust Book", ResourceType::Ebook))
                .await
                .expect("create resource");
            assert_eq!(created.title, "Rust Book");

            let listed = repository.list().await.expect("list resources");
            assert_eq!(listed.len(), 1);

            let searched = repository.search("rust").await.expect("search resources");
            assert_eq!(searched.len(), 1);

            let fetched = repository
                .get_by_id(created.id)
                .await
                .expect("get resource by id");
            assert_eq!(fetched.id, created.id);

            let duplicate = repository
                .create(new_resource("Rust Book", ResourceType::Ebook))
                .await;
            assert!(matches!(duplicate, Err(DomainError::Conflict(_))));

            let empty_query = repository.search(" ").await;
            assert!(matches!(empty_query, Err(DomainError::ValidationError(_))));

            let updated = repository
                .update(
                    created.id,
                    UpdateResource {
                        title: Some("Rust Book 2nd Edition".to_string()),
                        notes: Some("updated".to_string()),
                    },
                )
                .await
                .expect("update resource");
            assert_eq!(updated.title, "Rust Book 2nd Edition");
            assert_eq!(updated.notes.as_deref(), Some("updated"));

            repository
                .delete(created.id)
                .await
                .expect("delete resource");
            let deleted_get = repository.get_by_id(created.id).await;
            assert!(matches!(deleted_get, Err(DomainError::NotFound(_))));
            let delete_again = repository.delete(created.id).await;
            assert!(matches!(delete_again, Err(DomainError::NotFound(_))));
        });
    }

    #[test]
    fn ebook_meta_repository_upsert_and_get_contract() {
        let repository = InMemoryEbookMetaRepository {
            items: Mutex::new(HashMap::new()),
        };
        let resource_id = Uuid::new_v4();

        block_on(async {
            let missing = repository.get(resource_id).await;
            assert!(matches!(missing, Err(DomainError::NotFound(_))));

            let saved = repository
                .upsert(
                    resource_id,
                    NewEbookMeta {
                        author: Some("Author".to_string()),
                        isbn: Some("123".to_string()),
                        publisher: Some("Publisher".to_string()),
                        language: Some("en".to_string()),
                        file_format: Some("epub".to_string()),
                    },
                )
                .await
                .expect("upsert ebook meta");
            assert_eq!(saved.author.as_deref(), Some("Author"));

            let fetched = repository.get(resource_id).await.expect("get ebook meta");
            assert_eq!(fetched.isbn.as_deref(), Some("123"));
        });
    }

    #[test]
    fn web_reader_meta_repository_upsert_and_get_contract() {
        let repository = InMemoryWebReaderMetaRepository {
            items: Mutex::new(HashMap::new()),
        };
        let resource_id = Uuid::new_v4();

        block_on(async {
            let missing = repository.get(resource_id).await;
            assert!(matches!(missing, Err(DomainError::NotFound(_))));

            let saved = repository
                .upsert(
                    resource_id,
                    NewWebReaderMeta {
                        url: "https://example.com/reader".to_string(),
                        site_name: Some("Example".to_string()),
                        last_checked_chapter: Some("chapter 2".to_string()),
                        check_interval_secs: None,
                        last_checked_at: None,
                        progress_css_selector: None,
                    },
                )
                .await
                .expect("upsert web reader meta");
            assert_eq!(saved.site_name.as_deref(), Some("Example"));

            let fetched = repository
                .get(resource_id)
                .await
                .expect("get web reader meta");
            assert_eq!(fetched.url, "https://example.com/reader");
        });
    }

    #[test]
    fn location_repository_add_list_remove_and_error_contract() {
        let repository = InMemoryLocationRepository {
            items: Mutex::new(HashMap::new()),
        };
        let resource_id = Uuid::new_v4();

        block_on(async {
            let empty = repository.list(resource_id).await.expect("list locations");
            assert!(empty.is_empty());

            let created = repository
                .add(
                    resource_id,
                    NewResourceLocation {
                        device_id: "device-1".to_string(),
                        path_or_url: "/mnt/books/rust.epub".to_string(),
                        storage_type: StorageType::LocalFs,
                    },
                )
                .await
                .expect("add location");
            assert_eq!(created.device_id, "device-1");

            let listed = repository
                .list(resource_id)
                .await
                .expect("list created locations");
            assert_eq!(listed.len(), 1);

            repository
                .remove(resource_id, created.id)
                .await
                .expect("remove location");
            let removed = repository
                .list(resource_id)
                .await
                .expect("list locations after remove");
            assert!(removed.is_empty());

            let remove_again = repository.remove(resource_id, created.id).await;
            assert!(matches!(remove_again, Err(DomainError::NotFound(_))));
        });
    }

    // ---- Phase 2 in-memory implementations ----

    struct InMemoryChapterCheckRepository {
        items: Mutex<Vec<ChapterCheck>>,
    }

    #[async_trait]
    impl ChapterCheckRepository for InMemoryChapterCheckRepository {
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
            self.items.lock().expect("chapter check lock").push(check.clone());
            Ok(check)
        }

        async fn list(&self, resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError> {
            let guard = self.items.lock().expect("chapter check lock");
            let mut results: Vec<ChapterCheck> = guard
                .iter()
                .filter(|c| c.resource_id == resource_id)
                .cloned()
                .collect();
            results.sort_by(|a, b| b.checked_at.cmp(&a.checked_at));
            Ok(results)
        }
    }

    struct InMemoryNotificationRepository {
        items: Mutex<Vec<Notification>>,
    }

    #[async_trait]
    impl NotificationRepository for InMemoryNotificationRepository {
        async fn create(
            &self,
            resource_id: Uuid,
            message: String,
        ) -> Result<Notification, DomainError> {
            let notification = Notification {
                id: Uuid::new_v4(),
                resource_id,
                message,
                created_at: Utc::now(),
                read: false,
            };
            self.items
                .lock()
                .expect("notification lock")
                .push(notification.clone());
            Ok(notification)
        }

        async fn list(&self, unread_only: bool) -> Result<Vec<Notification>, DomainError> {
            let guard = self.items.lock().expect("notification lock");
            Ok(guard
                .iter()
                .filter(|n| !unread_only || !n.read)
                .cloned()
                .collect())
        }

        async fn mark_read(&self, id: Uuid) -> Result<(), DomainError> {
            let mut guard = self.items.lock().expect("notification lock");
            let notification = guard
                .iter_mut()
                .find(|n| n.id == id)
                .ok_or_else(|| DomainError::NotFound(format!("notification {id} not found")))?;
            notification.read = true;
            Ok(())
        }
    }

    #[test]
    fn chapter_check_repository_create_list_contract() {
        let repo = InMemoryChapterCheckRepository {
            items: Mutex::new(Vec::new()),
        };
        let resource_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        block_on(async {
            let empty = repo.list(resource_id).await.expect("list empty");
            assert!(empty.is_empty());

            let check = repo
                .create(resource_id, true, Some("ch-5".to_string()), None)
                .await
                .expect("create check");
            assert_eq!(check.resource_id, resource_id);
            assert!(check.has_new_chapter);
            assert_eq!(check.latest_chapter.as_deref(), Some("ch-5"));
            assert!(check.error_message.is_none());

            let _ = repo
                .create(resource_id, false, Some("ch-5".to_string()), None)
                .await
                .expect("create second check");

            let checks = repo.list(resource_id).await.expect("list checks");
            assert_eq!(checks.len(), 2);

            let other_checks = repo.list(other_id).await.expect("list other checks");
            assert!(other_checks.is_empty());

            let error_check = repo
                .create(resource_id, false, None, Some("timeout".to_string()))
                .await
                .expect("create error check");
            assert_eq!(error_check.error_message.as_deref(), Some("timeout"));
        });
    }

    #[test]
    fn chapter_check_serde_round_trip() {
        let now = Utc::now();
        let check = ChapterCheck {
            id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            has_new_chapter: true,
            latest_chapter: Some("Chapter 10".to_string()),
            checked_at: now,
            error_message: None,
        };
        let serialized = serde_json::to_string(&check).expect("serialize chapter check");
        let deserialized: ChapterCheck =
            serde_json::from_str(&serialized).expect("deserialize chapter check");
        assert_eq!(check.id, deserialized.id);
        assert_eq!(check.has_new_chapter, deserialized.has_new_chapter);
        assert_eq!(check.latest_chapter, deserialized.latest_chapter);
    }

    #[test]
    fn notification_repository_create_list_mark_read_contract() {
        let repo = InMemoryNotificationRepository {
            items: Mutex::new(Vec::new()),
        };
        let resource_id = Uuid::new_v4();

        block_on(async {
            let all_empty = repo.list(false).await.expect("list all empty");
            assert!(all_empty.is_empty());

            let n1 = repo
                .create(resource_id, "New chapter: ch-1".to_string())
                .await
                .expect("create notification");
            assert_eq!(n1.resource_id, resource_id);
            assert!(!n1.read);

            let n2 = repo
                .create(resource_id, "New chapter: ch-2".to_string())
                .await
                .expect("create second notification");

            let all = repo.list(false).await.expect("list all");
            assert_eq!(all.len(), 2);

            let unread = repo.list(true).await.expect("list unread");
            assert_eq!(unread.len(), 2);

            repo.mark_read(n1.id).await.expect("mark n1 read");

            let after_mark = repo.list(true).await.expect("list unread after mark");
            assert_eq!(after_mark.len(), 1);
            assert_eq!(after_mark[0].id, n2.id);

            let all_after = repo.list(false).await.expect("list all after mark");
            assert_eq!(all_after.len(), 2);

            let not_found = repo.mark_read(Uuid::new_v4()).await;
            assert!(matches!(not_found, Err(DomainError::NotFound(_))));
        });
    }

    #[test]
    fn notification_serde_round_trip() {
        let now = Utc::now();
        let notification = Notification {
            id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            message: "New chapter detected".to_string(),
            created_at: now,
            read: false,
        };
        let serialized = serde_json::to_string(&notification).expect("serialize notification");
        let deserialized: Notification =
            serde_json::from_str(&serialized).expect("deserialize notification");
        assert_eq!(notification.id, deserialized.id);
        assert_eq!(notification.message, deserialized.message);
        assert!(!deserialized.read);
    }
}
