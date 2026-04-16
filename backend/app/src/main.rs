mod bootstrap;
mod config;
mod runtime;

use std::path::Path;

use bootstrap::{bootstrap_api_key, ApiKeyBootstrapResult};
use config::AppConfig;
use runtime::build_app_router;

#[tokio::main]
async fn main() {
    let _ = domain::domain_ready();
    let _ = use_cases::use_cases_ready();
    let _ = plugins::plugins_ready();
    let _ = services::services_ready();
    let _ = adapters::adapters_ready();
    let _ = infrastructure::infrastructure_ready();

    let config = match AppConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to load config: {error:?}");
            std::process::exit(1);
        }
    };

    match bootstrap_api_key(config.api_key.clone(), Path::new(".env")) {
        ApiKeyBootstrapResult::Ready { api_key } => {
            let router = match build_app_router(&config, api_key) {
                Ok(router) => router,
                Err(error) => {
                    eprintln!("failed to build app router: {error:?}");
                    std::process::exit(1);
                }
            };
            let listener = match tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await {
                Ok(listener) => listener,
                Err(error) => {
                    eprintln!("failed to bind server listener: {error}");
                    std::process::exit(1);
                }
            };
            if let Err(error) = axum::serve(listener, router).await {
                eprintln!("server runtime failed: {error}");
                std::process::exit(1);
            }
        }
        ApiKeyBootstrapResult::GeneratedAndMustExit {
            api_key,
            env_path,
            env_write,
        } => {
            eprintln!(
                "API_KEY missing. Generated one-time key: {api_key}. Wrote to {} with outcome {:?}. Restart required.",
                env_path.display(),
                env_write
            );
            std::process::exit(1);
        }
    }
}
