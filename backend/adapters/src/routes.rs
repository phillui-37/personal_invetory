use std::sync::Arc;

use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    auth_middleware,
    batch_import::batch_import_ebooks,
    chapter_check::{list_chapter_checks, trigger_chapter_check},
    ebook::{
        add_ebook, add_ebook_location, delete_ebook, ebook_detail, list_ebooks,
        remove_ebook_location, search_ebooks, update_ebook,
    },
    notifications::{list_notifications, mark_notification_read, notifications_stream},
    system::{health, openapi},
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
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}
