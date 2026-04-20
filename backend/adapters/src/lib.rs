mod auth;
mod batch_import;
mod batch_update;
mod chapter_check;
mod dedup_handler;
mod device_handler;
mod ebook;
mod error;
mod game;
mod image;
mod retry_middleware;
mod tag_filter;
mod notifications;
mod openapi;
mod progress_handler;
mod routes;
mod search_options;
mod state;
mod sync_handler;
mod system;
mod tag_handler;
mod vault_handler;
mod video;
mod web_reader;

pub use auth::auth_middleware;
pub use error::ApiError;
pub use openapi::generate_openapi_json;
pub use retry_middleware::{classify_error, calculate_backoff, ErrorClass, RetryConfig};
pub use routes::build_router;
pub use search_options::{SearchFacets, SearchOptionsQuery, SortField, SortOrder};
pub use state::{AppState, BroadcastNotifier, NotificationEvent};

pub fn adapters_ready() -> bool {
    domain::domain_ready() && services::services_ready() && plugins::plugins_ready()
}
