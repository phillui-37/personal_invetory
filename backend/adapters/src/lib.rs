mod auth;
mod batch_import;
mod chapter_check;
mod dedup_handler;
mod ebook;
mod error;
mod game;
mod image;
mod notifications;
mod openapi;
mod routes;
mod state;
mod sync_handler;
mod system;
mod vault_handler;
mod video;
mod web_reader;

pub use auth::auth_middleware;
pub use error::ApiError;
pub use openapi::generate_openapi_json;
pub use routes::build_router;
pub use state::{AppState, BroadcastNotifier, NotificationEvent};

pub fn adapters_ready() -> bool {
    domain::domain_ready() && services::services_ready() && plugins::plugins_ready()
}
