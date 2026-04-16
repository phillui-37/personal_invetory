mod auth;
mod ebook;
mod error;
mod routes;
mod state;
mod system;
mod web_reader;

pub use auth::auth_middleware;
pub use error::ApiError;
pub use routes::build_router;
pub use state::AppState;

pub fn adapters_ready() -> bool {
    domain::domain_ready() && services::services_ready() && plugins::plugins_ready()
}
