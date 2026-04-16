use std::sync::Arc;

use adapters::{build_router, AppState};
use axum::Router;
use infrastructure::{resolve_search_strategy, AdapterFactory};
use services::{EbookService, SearchConfig, WebReaderService};

use crate::config::AppConfig;

pub fn build_app_router(config: &AppConfig, api_key: String) -> Result<Router, domain::DomainError> {
    let bundle = AdapterFactory::from_url(&config.database_url)?;
    let strategy = resolve_search_strategy(&bundle.database, None);
    let search_config = SearchConfig { strategy };

    let resource_repo = bundle.resource_repo;
    let location_repo = bundle.location_repo;

    let ebook_service = Arc::new(EbookService::new_with_search_config(
        resource_repo.clone(),
        bundle.ebook_meta_repo,
        location_repo.clone(),
        search_config,
    ));
    let web_reader_service = Arc::new(WebReaderService::new_with_search_config(
        resource_repo,
        bundle.web_reader_meta_repo,
        location_repo,
        search_config,
    ));

    let state = Arc::new(AppState::new(ebook_service, web_reader_service, api_key));
    Ok(build_router(state))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::build_app_router;
    use crate::config::AppConfig;

    fn test_config() -> AppConfig {
        AppConfig {
            database_url: "sqlite://:memory:".to_string(),
            api_key: Some("secret".to_string()),
            host: "127.0.0.1".to_string(),
            port: 8080,
            plugins_config: "plugins.toml".to_string(),
        }
    }

    #[tokio::test]
    async fn app_router_exposes_health_and_protects_inventory_routes() {
        let app = build_app_router(&test_config(), "secret".to_string()).expect("build router");

        let health_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/system/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(health_response.status(), StatusCode::OK);

        let protected_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/inventory/ebooks/list")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(protected_response.status(), StatusCode::UNAUTHORIZED);
    }
}
