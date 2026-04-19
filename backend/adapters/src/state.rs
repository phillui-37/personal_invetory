use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use domain::{
    ChapterCheck, DomainError, EbookMeta, EbookMetaRepository, GameMeta, GameMetaRepository,
    ImageMeta, ImageMetaRepository, LocationRepository, NewEbookMeta, NewGameMeta, NewImageMeta,
    NewResource, NewResourceLocation, NewVideoMeta, NewWebReaderMeta, Notification, Resource,
    ResourceLocation, ResourceRepository, UpdateResource, VideoMeta, VideoMetaRepository,
    WebReaderMeta, WebReaderMetaRepository,
};
use plugins::PluginRegistry;
use serde::Serialize;
use services::{
    ChapterCheckOps, EbookService, GameService, ImageService, SyncService, VideoService,
    WebReaderService,
};
use tokio::sync::broadcast;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct NotificationEvent {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub message: String,
}

pub struct AppState {
    pub ebook_service: Arc<EbookService>,
    pub web_reader_service: Arc<WebReaderService>,
    pub image_service: Arc<ImageService>,
    pub video_service: Arc<VideoService>,
    pub game_service: Arc<GameService>,
    pub chapter_check_service: Option<Arc<dyn ChapterCheckOps>>,
    pub plugin_registry: Arc<PluginRegistry>,
    pub api_key: String,
    pub openapi_json: String,
    pub notification_tx: broadcast::Sender<NotificationEvent>,
    pub vault_service: Option<Arc<dyn domain::vault::CredentialVault>>,
    pub dedup_service: Option<Arc<services::DedupService>>,
    pub sync_service: Option<Arc<SyncService>>,
}

impl AppState {
    pub fn new(
        ebook_service: Arc<EbookService>,
        web_reader_service: Arc<WebReaderService>,
        image_service: Arc<ImageService>,
        video_service: Arc<VideoService>,
        game_service: Arc<GameService>,
        api_key: String,
    ) -> Self {
        let (notification_tx, _) = broadcast::channel(64);
        Self {
            ebook_service,
            web_reader_service,
            image_service,
            video_service,
            game_service,
            chapter_check_service: None,
            plugin_registry: Arc::new(PluginRegistry::default()),
            api_key,
            openapi_json: "{}".to_string(),
            notification_tx,
            vault_service: None,
            dedup_service: None,
            sync_service: None,
        }
    }

    pub fn with_chapter_check_service(mut self, svc: Arc<dyn ChapterCheckOps>) -> Self {
        self.chapter_check_service = Some(svc);
        self
    }

    pub fn with_vault_service(mut self, svc: Arc<dyn domain::vault::CredentialVault>) -> Self {
        self.vault_service = Some(svc);
        self
    }

    pub fn with_dedup_service(mut self, svc: Arc<services::DedupService>) -> Self {
        self.dedup_service = Some(svc);
        self
    }

    pub fn with_sync_service(mut self, svc: Arc<SyncService>) -> Self {
        self.sync_service = Some(svc);
        self
    }

    pub fn for_tests(api_key: String) -> Self {
        let resource_repo = Arc::new(NoopResourceRepository::default());
        let location_repo = Arc::new(NoopLocationRepository::default());

        let ebook_service = Arc::new(EbookService::new(
            resource_repo.clone(),
            Arc::new(NoopEbookMetaRepository::default()),
            location_repo.clone(),
        ));
        let web_reader_service = Arc::new(WebReaderService::new(
            resource_repo.clone(),
            Arc::new(NoopWebReaderMetaRepository::default()),
            location_repo.clone(),
        ));
        let image_service = Arc::new(ImageService::new(
            resource_repo.clone(),
            Arc::new(NoopImageMetaRepository::default()),
            location_repo.clone(),
        ));
        let video_service = Arc::new(VideoService::new(
            resource_repo.clone(),
            Arc::new(NoopVideoMetaRepository::default()),
            location_repo.clone(),
        ));
        let game_service = Arc::new(GameService::new(
            resource_repo,
            Arc::new(NoopGameMetaRepository::default()),
            location_repo,
        ));

        let mut state = Self::new(
            ebook_service,
            web_reader_service,
            image_service,
            video_service,
            game_service,
            api_key,
        );
        state.chapter_check_service = Some(Arc::new(NoopChapterCheckOps));
        state.vault_service = None;
        state.dedup_service = None;
        state.sync_service = None;
        state
    }
}

struct NoopChapterCheckOps;

#[async_trait]
impl ChapterCheckOps for NoopChapterCheckOps {
    async fn check_resource(&self, resource_id: Uuid) -> Result<ChapterCheck, DomainError> {
        Err(DomainError::NotFound(format!("resource {resource_id} not found")))
    }

    async fn list_check_history(&self, _resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError> {
        Ok(Vec::new())
    }

    async fn list_notifications(&self, _unread_only: bool) -> Result<Vec<Notification>, DomainError> {
        Ok(Vec::new())
    }

    async fn mark_notification_read(&self, _id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }
}

#[derive(Default)]
struct NoopResourceRepository;

#[async_trait]
impl ResourceRepository for NoopResourceRepository {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        Ok(Vec::new())
    }

    async fn search(&self, _query: &str) -> Result<Vec<Resource>, DomainError> {
        Ok(Vec::new())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
        Err(DomainError::NotFound(format!("resource {id} not found")))
    }

    async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
        let now = Utc::now();
        Ok(Resource {
            id: Uuid::new_v4(),
            title: input.title,
            notes: input.notes,
            resource_type: input.resource_type,
            created_at: now,
            updated_at: now,
        })
    }

    async fn update(&self, id: Uuid, _input: UpdateResource) -> Result<Resource, DomainError> {
        Err(DomainError::NotFound(format!("resource {id} not found")))
    }

    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }
}

#[derive(Default)]
struct NoopEbookMetaRepository;

#[async_trait]
impl EbookMetaRepository for NoopEbookMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        Err(DomainError::NotFound(format!(
            "ebook meta for {resource_id} not found"
        )))
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewEbookMeta,
    ) -> Result<EbookMeta, DomainError> {
        Ok(EbookMeta {
            resource_id,
            author: input.author,
            isbn: input.isbn,
            publisher: input.publisher,
            language: input.language,
            file_format: input.file_format,
        })
    }
}

#[derive(Default)]
struct NoopWebReaderMetaRepository;

#[async_trait]
impl WebReaderMetaRepository for NoopWebReaderMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
        Err(DomainError::NotFound(format!(
            "web reader meta for {resource_id} not found"
        )))
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewWebReaderMeta,
    ) -> Result<WebReaderMeta, DomainError> {
        Ok(WebReaderMeta {
            resource_id,
            url: input.url,
            site_name: input.site_name,
            last_checked_chapter: input.last_checked_chapter,
            check_interval_secs: input.check_interval_secs,
            last_checked_at: input.last_checked_at,
            progress_css_selector: input.progress_css_selector,
        })
    }
}

#[derive(Default)]
struct NoopLocationRepository {
    _keep: HashMap<Uuid, Vec<ResourceLocation>>,
}

#[async_trait]
impl LocationRepository for NoopLocationRepository {
    async fn list(&self, _resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        Ok(Vec::new())
    }

    async fn add(
        &self,
        resource_id: Uuid,
        input: NewResourceLocation,
    ) -> Result<ResourceLocation, DomainError> {
        Ok(ResourceLocation {
            id: Uuid::new_v4(),
            resource_id,
            device_id: input.device_id,
            path_or_url: input.path_or_url,
            storage_type: input.storage_type,
        })
    }

    async fn remove(&self, _resource_id: Uuid, _location_id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }
}

/// Implements NotificationBroadcaster by sending to a tokio broadcast channel.
pub struct BroadcastNotifier(pub broadcast::Sender<NotificationEvent>);

impl domain::NotificationBroadcaster for BroadcastNotifier {
    fn broadcast(&self, notification: &Notification) {
        let event = NotificationEvent {
            id: notification.id,
            resource_id: notification.resource_id,
            message: notification.message.clone(),
        };
        let _ = self.0.send(event);
    }
}

#[derive(Default)]
struct NoopImageMetaRepository;

#[async_trait]
impl ImageMetaRepository for NoopImageMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<ImageMeta, DomainError> {
        Err(DomainError::NotFound(format!(
            "image meta for {resource_id} not found"
        )))
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewImageMeta,
    ) -> Result<ImageMeta, DomainError> {
        Ok(ImageMeta {
            resource_id,
            width: input.width,
            height: input.height,
            file_format: input.file_format,
            file_size_bytes: input.file_size_bytes,
        })
    }
}

#[derive(Default)]
struct NoopVideoMetaRepository;

#[async_trait]
impl VideoMetaRepository for NoopVideoMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<VideoMeta, DomainError> {
        Err(DomainError::NotFound(format!(
            "video meta for {resource_id} not found"
        )))
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewVideoMeta,
    ) -> Result<VideoMeta, DomainError> {
        Ok(VideoMeta {
            resource_id,
            duration_secs: input.duration_secs,
            file_format: input.file_format,
            resolution: input.resolution,
            file_size_bytes: input.file_size_bytes,
        })
    }
}

#[derive(Default)]
struct NoopGameMetaRepository;

#[async_trait]
impl GameMetaRepository for NoopGameMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<GameMeta, DomainError> {
        Err(DomainError::NotFound(format!(
            "game meta for {resource_id} not found"
        )))
    }

    async fn upsert(
        &self,
        resource_id: Uuid,
        input: NewGameMeta,
    ) -> Result<GameMeta, DomainError> {
        Ok(GameMeta {
            resource_id,
            platform: input.platform,
            store: input.store,
            developer: input.developer,
            publisher: input.publisher,
            manual_notes: input.manual_notes,
        })
    }
}
