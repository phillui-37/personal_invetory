use std::sync::Arc;

use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::{
    auth_middleware,
    batch_import::batch_import_ebooks,
    chapter_check::{list_chapter_checks, trigger_chapter_check},
    dedup_handler::{dismiss_warning, list_pending_warnings, merge_resources, scan_duplicates},
    device_handler::{current_device, delink_device, list_devices, register_device},
    ebook::{
        add_ebook, add_ebook_location, delete_ebook, ebook_detail, list_ebooks,
        remove_ebook_location, search_ebooks, update_ebook,
    },
    game::{
        add_game, add_game_location, delete_game, game_detail, list_games, remove_game_location,
        search_games, update_game,
    },
    image::{
        add_image, add_image_location, delete_image, image_detail, list_images,
        remove_image_location, search_images, update_image,
    },
    notifications::{list_notifications, mark_notification_read, notifications_stream},
    progress_handler::{
        get_ebook_progress, get_game_progress, get_image_progress, get_video_progress,
        get_web_reader_progress, patch_ebook_progress, patch_game_progress, patch_image_progress,
        patch_video_progress, patch_web_reader_progress,
    },
    sync_handler::{ecosystem_status, get_sync_job, list_platform_syncs, trigger_sync},
    system::{health, openapi},
    vault_handler::{
        delete_credential, initialize_vault, list_vault_platforms, lock_vault,
        retrieve_credential, store_credential, unlock_vault, vault_status,
    },
    video::{
        add_video, add_video_location, delete_video, list_videos, remove_video_location,
        search_videos, update_video, video_detail,
    },
    web_reader::{
        add_web_reader, add_web_reader_location, delete_web_reader, list_web_readers,
        remove_web_reader_location, search_web_readers, update_web_reader, web_reader_detail,
    },
    AppState,
};

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/system/health", get(health))
        .route("/api/v1/system/openapi", get(openapi))
        .route("/api/v1/inventory/ebooks/list", get(list_ebooks))
        .route("/api/v1/inventory/ebooks/search", get(search_ebooks))
        .route("/api/v1/inventory/ebooks/add", post(add_ebook))
        .route("/api/v1/inventory/ebooks/batch-import", post(batch_import_ebooks))
        .route("/api/v1/inventory/ebooks/:id/detail", get(ebook_detail))
        .route("/api/v1/inventory/ebooks/:id/update", put(update_ebook))
        .route("/api/v1/inventory/ebooks/:id/delete", delete(delete_ebook))
        .route(
            "/api/v1/inventory/ebooks/:id/locations/add",
            post(add_ebook_location),
        )
        .route(
            "/api/v1/inventory/ebooks/:id/locations/:loc_id/remove",
            delete(remove_ebook_location),
        )
        .route("/api/v1/inventory/web-readers/list", get(list_web_readers))
        .route(
            "/api/v1/inventory/web-readers/search",
            get(search_web_readers),
        )
        .route("/api/v1/inventory/web-readers/add", post(add_web_reader))
        .route(
            "/api/v1/inventory/web-readers/:id/detail",
            get(web_reader_detail),
        )
        .route(
            "/api/v1/inventory/web-readers/:id/update",
            put(update_web_reader),
        )
        .route(
            "/api/v1/inventory/web-readers/:id/delete",
            delete(delete_web_reader),
        )
        .route(
            "/api/v1/inventory/web-readers/:id/locations/add",
            post(add_web_reader_location),
        )
        .route(
            "/api/v1/inventory/web-readers/:id/locations/:loc_id/remove",
            delete(remove_web_reader_location),
        )
        .route("/api/v1/inventory/images/list", get(list_images))
        .route("/api/v1/inventory/images/search", get(search_images))
        .route("/api/v1/inventory/images/add", post(add_image))
        .route(
            "/api/v1/inventory/images/:id/detail",
            get(image_detail),
        )
        .route(
            "/api/v1/inventory/images/:id/update",
            put(update_image),
        )
        .route(
            "/api/v1/inventory/images/:id/delete",
            delete(delete_image),
        )
        .route(
            "/api/v1/inventory/images/:id/locations/add",
            post(add_image_location),
        )
        .route(
            "/api/v1/inventory/images/:id/locations/:loc_id/remove",
            delete(remove_image_location),
        )
        .route("/api/v1/inventory/videos/list", get(list_videos))
        .route("/api/v1/inventory/videos/search", get(search_videos))
        .route("/api/v1/inventory/videos/add", post(add_video))
        .route(
            "/api/v1/inventory/videos/:id/detail",
            get(video_detail),
        )
        .route(
            "/api/v1/inventory/videos/:id/update",
            put(update_video),
        )
        .route(
            "/api/v1/inventory/videos/:id/delete",
            delete(delete_video),
        )
        .route(
            "/api/v1/inventory/videos/:id/locations/add",
            post(add_video_location),
        )
        .route(
            "/api/v1/inventory/videos/:id/locations/:loc_id/remove",
            delete(remove_video_location),
        )
        .route("/api/v1/inventory/games/list", get(list_games))
        .route("/api/v1/inventory/games/search", get(search_games))
        .route("/api/v1/inventory/games/add", post(add_game))
        .route(
            "/api/v1/inventory/games/:id/detail",
            get(game_detail),
        )
        .route(
            "/api/v1/inventory/games/:id/update",
            put(update_game),
        )
        .route(
            "/api/v1/inventory/games/:id/delete",
            delete(delete_game),
        )
        .route(
            "/api/v1/inventory/games/:id/locations/add",
            post(add_game_location),
        )
        .route(
            "/api/v1/inventory/games/:id/locations/:loc_id/remove",
            delete(remove_game_location),
        )
        .route(
            "/api/v1/inventory/web-readers/:id/check",
            post(trigger_chapter_check),
        )
        .route(
            "/api/v1/inventory/web-readers/:id/checks",
            get(list_chapter_checks),
        )
        .route("/api/v1/notifications", get(list_notifications))
        .route("/api/v1/notifications/:id/read", post(mark_notification_read))
        .route("/api/v1/notifications/stream", get(notifications_stream))
        // Vault
        .route("/api/v1/vault/status", get(vault_status))
        .route("/api/v1/vault/initialize", post(initialize_vault))
        .route("/api/v1/vault/unlock", post(unlock_vault))
        .route("/api/v1/vault/lock", post(lock_vault))
        .route("/api/v1/vault/credentials/store", post(store_credential))
        .route("/api/v1/vault/credentials/retrieve", post(retrieve_credential))
        .route("/api/v1/vault/credentials/delete", post(delete_credential))
        .route("/api/v1/vault/platforms", get(list_vault_platforms))
        // Dedup
        .route("/api/v1/dedup/scan", post(scan_duplicates))
        .route("/api/v1/dedup/warnings", get(list_pending_warnings))
        .route("/api/v1/dedup/warnings/:id/dismiss", post(dismiss_warning))
        .route("/api/v1/dedup/warnings/:id/merge", post(merge_resources))
        // Ecosystem sync
        .route("/api/v1/ecosystem/status", get(ecosystem_status))
        .route("/api/v1/ecosystem/:platform/sync", post(trigger_sync))
        .route("/api/v1/ecosystem/:platform/syncs", get(list_platform_syncs))
        .route("/api/v1/ecosystem/:platform/syncs/:id", get(get_sync_job))
        // Device management
        .route("/api/v1/devices", get(list_devices))
        .route("/api/v1/devices/current", get(current_device))
        .route("/api/v1/devices/register", post(register_device))
        .route("/api/v1/devices/:device_id/delink", post(delink_device))
        // Progress
        .route(
            "/api/v1/inventory/ebooks/:id/progress",
            get(get_ebook_progress).patch(patch_ebook_progress),
        )
        .route(
            "/api/v1/inventory/web-readers/:id/progress",
            get(get_web_reader_progress).patch(patch_web_reader_progress),
        )
        .route(
            "/api/v1/inventory/images/:id/progress",
            get(get_image_progress).patch(patch_image_progress),
        )
        .route(
            "/api/v1/inventory/videos/:id/progress",
            get(get_video_progress).patch(patch_video_progress),
        )
        .route(
            "/api/v1/inventory/games/:id/progress",
            get(get_game_progress).patch(patch_game_progress),
        )
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}
