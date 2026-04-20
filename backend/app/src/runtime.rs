use std::sync::Arc;
use std::{fs, io::ErrorKind};

use adapters::{build_router, generate_openapi_json, AppState, BroadcastNotifier};
use axum::Router;
use infrastructure::{resolve_search_strategy, AdapterFactory};
use plugins::{PluginRegistry, PluginsConfig, PluginsToml, WebChecker, WebCheckerConfig};
use services::{
    ChapterCheckService, DedupService, DeviceService, EbookService, GameService, ImageService,
    OtpInteractionService, ProgressService, SearchConfig, SyncService, TagService, VaultService, VideoService,
    WebReaderService,
};
use tokio_util::sync::CancellationToken;

use crate::config::AppConfig;
use crate::scheduler::{OpsCheckRunner, Scheduler};

pub async fn build_app_router(
    config: &AppConfig,
    api_key: String,
) -> Result<Router, domain::DomainError> {
    let bundle = AdapterFactory::from_url(&config.database_url).await?;
    let strategy = resolve_search_strategy(&bundle.database, None);
    let search_config = SearchConfig { strategy };

    let resource_repo = bundle.resource_repo;
    let location_repo = bundle.location_repo;
    let web_reader_meta_repo = bundle.web_reader_meta_repo;
    let chapter_check_repo = bundle.chapter_check_repo;
    let notification_repo = bundle.notification_repo;

    let ebook_service = Arc::new(EbookService::new_with_search_config(
        resource_repo.clone(),
        bundle.ebook_meta_repo.clone(),
        location_repo.clone(),
        search_config,
    ));
    let web_reader_service = Arc::new(WebReaderService::new_with_search_config(
        resource_repo.clone(),
        web_reader_meta_repo.clone(),
        location_repo.clone(),
        search_config,
    ));
    let image_service = Arc::new(ImageService::new_with_search_config(
        resource_repo.clone(),
        bundle.image_meta_repo.clone(),
        location_repo.clone(),
        search_config,
    ));
    let video_service = Arc::new(VideoService::new_with_search_config(
        resource_repo.clone(),
        bundle.video_meta_repo.clone(),
        location_repo.clone(),
        search_config,
    ));
    let game_service = Arc::new(GameService::new_with_search_config(
        resource_repo.clone(),
        bundle.game_meta_repo.clone(),
        location_repo.clone(),
        search_config,
    ));

    let vault_service: Arc<dyn domain::vault::CredentialVault> =
        Arc::new(VaultService::new(bundle.vault_backend));
    let dedup_service = Arc::new(DedupService::new(
        resource_repo.clone(),
        location_repo,
        bundle.dedup_warning_repo,
    ));
    let sync_service = Arc::new(SyncService::new(
        resource_repo.clone(),
        bundle.game_meta_repo.clone(),
        bundle.ebook_meta_repo.clone(),
        bundle.image_meta_repo.clone(),
        bundle.video_meta_repo.clone(),
        bundle.sync_job_repo,
    ));

    let hostname_str = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| config.device_id.clone());
    let effective_hostname = config.device_name.clone().unwrap_or(hostname_str);
    let device_service = std::sync::Arc::new(DeviceService::new(
        bundle.device_repo.clone(),
        config.device_id.clone(),
        effective_hostname,
    ));
    let progress_service = Arc::new(ProgressService::new(bundle.progress_repo.clone()));
    let tag_service = Arc::new(TagService::new(
        bundle.tag_repo.clone(),
        bundle.resource_tag_repo.clone(),
    ));
    let otp_service = Arc::new(OtpInteractionService::new(config.otp_timeout_secs.unwrap_or(300)));

    let mut state = AppState::new(
        ebook_service,
        web_reader_service,
        image_service,
        video_service,
        game_service,
        api_key,
    );
    let web_checker_config = load_web_checker_config(&config.plugins_config);
    let mut plugin_registry = PluginRegistry::from_config(&PluginsConfig {
        use_noop_metadata_extractor: false,
        use_noop_web_checker: cfg!(feature = "stub-plugins") && !cfg!(feature = "real-plugins"),
        web_checker: web_checker_config.clone(),
    });
    let checker: Arc<dyn WebChecker> = plugin_registry
        .web_checkers
        .pop()
        .ok_or_else(|| {
            domain::DomainError::InternalError("no web checker plugin configured".to_string())
        })?
        .into();
    let broadcaster = Arc::new(BroadcastNotifier(state.notification_tx.clone()));
    let chapter_check_service = Arc::new(
        ChapterCheckService::new(
            checker.clone(),
            chapter_check_repo,
            notification_repo,
            web_reader_meta_repo.clone(),
            None,
        )
        .with_broadcaster(broadcaster),
    );
    state = state.with_chapter_check_service(chapter_check_service.clone());
    state = state.with_vault_service(vault_service);
    state = state.with_dedup_service(dedup_service);
    state = state.with_sync_service(sync_service);
    state = state.with_device_service(device_service);
    state = state.with_progress_service(progress_service);
    state = state.with_tag_service(tag_service);
    state = state.with_otp_service(otp_service);
    state.plugin_registry = Arc::new(plugin_registry);
    state.openapi_json = generate_openapi_json();
    if config.scheduler_enabled {
        let scheduler = Scheduler::new(
            Arc::new(OpsCheckRunner(chapter_check_service)),
            web_reader_meta_repo,
            resource_repo,
            scheduler_default_interval(&web_checker_config),
            CancellationToken::new(),
        );
        scheduler.start();
    }
    let state = Arc::new(state);
    Ok(build_router(state))
}

fn load_web_checker_config(path: &str) -> WebCheckerConfig {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            eprintln!("plugins config not found at {path}; using defaults");
            return WebCheckerConfig::default();
        }
        Err(error) => {
            eprintln!("failed to read plugins config at {path}: {error}; using defaults");
            return WebCheckerConfig::default();
        }
    };

    let parsed = match toml::from_str::<PluginsToml>(&content) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("failed to parse plugins config at {path}: {error}; using defaults");
            return WebCheckerConfig::default();
        }
    };

    let config = parsed.web_checker.unwrap_or_default();
    for error in plugins::validate_site_config_patterns(&config.sites) {
        eprintln!("plugins config warning: {error}");
    }
    config
}

fn scheduler_default_interval(config: &WebCheckerConfig) -> std::time::Duration {
    std::time::Duration::from_secs(match config.default_interval_secs {
        0 => 3600,
        secs => secs,
    })
}

#[cfg(test)]
mod tests {
    use axum::{
        body::to_bytes,
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::{json, Value};
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
            chromium_path: None,
            fcm_service_account: None,
            scheduler_enabled: false,
            device_id: "test-device".to_string(),
            device_name: None,
            otp_timeout_secs: None,
        }
    }

    #[tokio::test]
    async fn app_router_exposes_health_and_protects_inventory_routes() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

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

    #[tokio::test]
    async fn app_router_wires_notifications_service() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/notifications")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_router_wires_image_service() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/inventory/images/list")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_router_wires_video_service() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/inventory/videos/list")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_router_wires_game_service() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/inventory/games/list")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_router_wires_vault_service() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/vault/status")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_router_wires_dedup_service() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/dedup/warnings")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_router_wires_progress_service() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/inventory/ebooks/add")
                    .header("authorization", "Bearer secret")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "title": "Progress Test",
                            "author": "Tester",
                            "file_format": "epub"
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(create_response.status(), StatusCode::OK);
        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create body");
        let created: Value = serde_json::from_slice(&create_body).expect("valid json");
        let resource_id = created["resource"]["id"]
            .as_str()
            .expect("resource id")
            .to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/inventory/ebooks/{resource_id}/progress"))
                    .header("authorization", "Bearer secret")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"progress": 0.5}).to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_router_wires_tag_service() {
        let app = build_app_router(&test_config(), "secret".to_string()).await.expect("build router");

        let tag_list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tags")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(tag_list_response.status(), StatusCode::OK);

        let filtered_list_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/inventory/ebooks/list?tag=sci-fi")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(filtered_list_response.status(), StatusCode::OK);
    }
}
