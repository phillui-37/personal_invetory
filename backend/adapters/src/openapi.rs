use domain::{
    ChapterCheck, EbookMeta, GameMeta, ImageMeta, Notification, Resource, ResourceLocation,
    ResourceType, StorageType, VideoMeta, WebReaderMeta,
};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::{
    batch_import::{
        BatchImportEbookEntry, BatchImportFailure, BatchImportLocationInput, BatchImportResponse,
        BatchImportSuccess,
    },
    device_handler::{DeviceResponse, RegisterDeviceRequest},
    ebook::{AddEbookRequest, AddLocationRequest as EbookAddLocationRequest, EbookDetailResponse, UpdateEbookRequest},
    game::{AddGameRequest, AddLocationRequest as GameAddLocationRequest, GameDetailResponse, UpdateGameRequest},
    image::{AddImageRequest, AddLocationRequest as ImageAddLocationRequest, ImageDetailResponse, UpdateImageRequest},
    progress_handler::{PatchProgressRequest, ProgressResponse},
    state::NotificationEvent,
    sync_handler::{
        DiscoveredItemInput, EcosystemStatusResponse, PlatformStatus, SyncJobListResponse,
        SyncJobResponse, TriggerSyncRequest, TriggerSyncResponse,
    },
    video::{AddLocationRequest as VideoAddLocationRequest, AddVideoRequest, UpdateVideoRequest, VideoDetailResponse},
    web_reader::{
        AddLocationRequest as WebReaderAddLocationRequest, UpsertWebReaderRequest,
        UpdateWebReaderRequest, WebReaderDetailResponse,
    },
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        crate::ebook::list_ebooks,
        crate::ebook::search_ebooks,
        crate::ebook::add_ebook,
        crate::ebook::ebook_detail,
        crate::ebook::update_ebook,
        crate::ebook::delete_ebook,
        crate::ebook::add_ebook_location,
        crate::ebook::remove_ebook_location,
        crate::web_reader::list_web_readers,
        crate::web_reader::search_web_readers,
        crate::web_reader::add_web_reader,
        crate::web_reader::web_reader_detail,
        crate::web_reader::update_web_reader,
        crate::web_reader::delete_web_reader,
        crate::web_reader::add_web_reader_location,
        crate::web_reader::remove_web_reader_location,
        crate::image::list_images,
        crate::image::search_images,
        crate::image::add_image,
        crate::image::image_detail,
        crate::image::update_image,
        crate::image::delete_image,
        crate::image::add_image_location,
        crate::image::remove_image_location,
        crate::video::list_videos,
        crate::video::search_videos,
        crate::video::add_video,
        crate::video::video_detail,
        crate::video::update_video,
        crate::video::delete_video,
        crate::video::add_video_location,
        crate::video::remove_video_location,
        crate::game::list_games,
        crate::game::search_games,
        crate::game::add_game,
        crate::game::game_detail,
        crate::game::update_game,
        crate::game::delete_game,
        crate::game::add_game_location,
        crate::game::remove_game_location,
        crate::chapter_check::trigger_chapter_check,
        crate::chapter_check::list_chapter_checks,
        crate::notifications::list_notifications,
        crate::notifications::mark_notification_read,
        crate::notifications::notifications_stream,
        crate::batch_import::batch_import_ebooks,
        crate::sync_handler::trigger_sync,
        crate::sync_handler::list_platform_syncs,
        crate::sync_handler::get_sync_job,
        crate::sync_handler::ecosystem_status,
        crate::device_handler::list_devices,
        crate::device_handler::current_device,
        crate::device_handler::register_device,
        crate::device_handler::delink_device,
        crate::progress_handler::get_progress,
        crate::progress_handler::patch_progress,
    ),
    components(
        schemas(
            Resource,
            ResourceType,
            ResourceLocation,
            StorageType,
            EbookMeta,
            WebReaderMeta,
            ImageMeta,
            VideoMeta,
            GameMeta,
            ChapterCheck,
            Notification,
            NotificationEvent,
            EbookDetailResponse,
            AddEbookRequest,
            UpdateEbookRequest,
            EbookAddLocationRequest,
            WebReaderDetailResponse,
            UpsertWebReaderRequest,
            UpdateWebReaderRequest,
            WebReaderAddLocationRequest,
            ImageDetailResponse,
            AddImageRequest,
            UpdateImageRequest,
            ImageAddLocationRequest,
            VideoDetailResponse,
            AddVideoRequest,
            UpdateVideoRequest,
            VideoAddLocationRequest,
            GameDetailResponse,
            AddGameRequest,
            UpdateGameRequest,
            GameAddLocationRequest,
            BatchImportEbookEntry,
            BatchImportLocationInput,
            BatchImportSuccess,
            BatchImportFailure,
            BatchImportResponse,
            SyncJobResponse,
            SyncJobListResponse,
            TriggerSyncRequest,
            TriggerSyncResponse,
            DiscoveredItemInput,
            EcosystemStatusResponse,
            PlatformStatus,
            DeviceResponse,
            RegisterDeviceRequest,
            ProgressResponse,
            PatchProgressRequest,
        )
    ),
    info(
        title = "Personal Inventory API",
        version = "1.0.0",
        description = "API for managing personal inventory resources (ebooks, web readers, images, videos, games)"
    )
)]
pub struct ApiDoc;

pub fn generate_openapi_json() -> String {
    ApiDoc::openapi()
        .to_pretty_json()
        .unwrap_or_else(|_| "{}".to_string())
}
