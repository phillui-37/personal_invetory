use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use chrono::Utc;
use domain::{
    device::{Device, DeviceRepository},
    progress::{ProgressRepository, ResourceProgress},
    tag::{ResourceTagRepository, Tag, TagRepository},
    DomainError, EbookMeta, EbookMetaRepository, GameMeta, GameMetaRepository, ImageMeta,
    ImageMetaRepository, LocationRepository, NewEbookMeta, NewGameMeta, NewImageMeta,
    NewResourceLocation, NewVideoMeta, NewWebReaderMeta, Resource, ResourceLocation,
    ResourceRepository, ResourceType, VideoMeta, VideoMetaRepository, WebReaderMeta,
    WebReaderMetaRepository,
};
use serde_json::{json, Value};
use services::{EbookService, GameService, ImageService, TagService, VideoService, WebReaderService};
use tower::ServiceExt;
use uuid::Uuid;

use adapters::{build_router, AppState};

fn app_with_openapi(openapi_json: &str) -> axum::Router {
    let mut state = AppState::for_tests("secret-key".to_string());
    state.openapi_json = openapi_json.to_string();
    build_router(Arc::new(state))
}

// ── In-memory DeviceRepository for handler tests ──────────────────────────

#[derive(Default)]
struct FakeHandlerDeviceRepo {
    devices: std::sync::Mutex<Vec<Device>>,
}

impl FakeHandlerDeviceRepo {
    fn empty() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn with_device(device_id: &str, device_name: Option<&str>) -> Arc<Self> {
        let repo = Self::empty();
        let now = Utc::now();
        repo.devices.lock().unwrap().push(Device {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            device_name: device_name.map(str::to_string),
            linked_at: now,
            delinked_at: None,
            location_count: 0,
        });
        repo
    }
}

#[async_trait]
impl DeviceRepository for FakeHandlerDeviceRepo {
    async fn all_with_counts(&self) -> Result<Vec<Device>, DomainError> {
        Ok(self.devices.lock().unwrap().clone())
    }

    async fn active_by_device_id(&self, device_id: &str) -> Result<Option<Device>, DomainError> {
        Ok(self
            .devices
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.device_id == device_id && d.delinked_at.is_none())
            .cloned())
    }

    async fn register(&self, device_id: &str, device_name: Option<&str>) -> Result<Device, DomainError> {
        let now = Utc::now();
        let device = Device {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            device_name: device_name.map(str::to_string),
            linked_at: now,
            delinked_at: None,
            location_count: 0,
        };
        self.devices.lock().unwrap().push(device.clone());
        Ok(device)
    }

    async fn delink(&self, device_id: &str, at: chrono::DateTime<Utc>) -> Result<(), DomainError> {
        let mut devices = self.devices.lock().unwrap();
        let d = devices
            .iter_mut()
            .find(|d| d.device_id == device_id && d.delinked_at.is_none())
            .ok_or_else(|| DomainError::NotFound(format!("device '{device_id}' not found")))?;
        d.delinked_at = Some(at);
        Ok(())
    }
}

fn app_with_device_service(device_id: &str) -> axum::Router {
    let openapi = r#"{"paths":{"/api/v1/system/health":{}}}"#;
    let repo = FakeHandlerDeviceRepo::empty();
    let svc = Arc::new(services::DeviceService::new(
        repo,
        device_id.to_string(),
        "test-host".to_string(),
    ));
    let mut state = AppState::for_tests("secret-key".to_string());
    state.openapi_json = openapi.to_string();
    let state = state.with_device_service(svc);
    build_router(Arc::new(state))
}

fn app_with_device_service_and_device(device_id: &str) -> axum::Router {
    let openapi = r#"{"paths":{"/api/v1/system/health":{}}}"#;
    let repo = FakeHandlerDeviceRepo::with_device(device_id, Some("Test Device"));
    let svc = Arc::new(services::DeviceService::new(
        repo,
        device_id.to_string(),
        "test-host".to_string(),
    ));
    let mut state = AppState::for_tests("secret-key".to_string());
    state.openapi_json = openapi.to_string();
    let state = state.with_device_service(svc);
    build_router(Arc::new(state))
}

#[tokio::test]
async fn ebook_handlers_cover_crud_and_location_routes() {
    let app = app_with_openapi(r#"{"paths":{"/api/v1/system/health":{}}}"#);

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/ebooks/list")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list.status(), StatusCode::OK);

    let add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/ebooks/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Book",
                        "notes": "n",
                        "author": "A",
                        "file_format": "epub"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add.status(), StatusCode::OK);

    let invalid_add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/ebooks/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"title": "Book", "file_format": "txt"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(invalid_add.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let missing_id = "550e8400-e29b-41d4-a716-446655440000";

    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/ebooks/{missing_id}/detail"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(detail.status(), StatusCode::NOT_FOUND);

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/inventory/ebooks/{missing_id}/update"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"title": "Updated"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(update.status(), StatusCode::NOT_FOUND);

    let delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/inventory/ebooks/{missing_id}/delete"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delete.status(), StatusCode::OK);

    let add_location = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/inventory/ebooks/{missing_id}/locations/add"
                ))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "device_id": "dev-1",
                        "path_or_url": "/books/book.epub",
                        "storage_type": "localfs"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add_location.status(), StatusCode::OK);

    let remove_location = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/inventory/ebooks/{missing_id}/locations/{missing_id}/remove"
                ))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(remove_location.status(), StatusCode::OK);
}

#[tokio::test]
async fn web_reader_handlers_cover_crud_and_location_routes() {
    let app = app_with_openapi(r#"{"paths":{"/api/v1/system/health":{}}}"#);
    let missing_id = "550e8400-e29b-41d4-a716-446655440000";

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/web-readers/list")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list.status(), StatusCode::OK);

    let search = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/web-readers/search?q=reader")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(search.status(), StatusCode::OK);

    let add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/web-readers/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"title": "Reader", "url": "https://example.com/ch1"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add.status(), StatusCode::OK);

    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/web-readers/{missing_id}/detail"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(detail.status(), StatusCode::NOT_FOUND);

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/inventory/web-readers/{missing_id}/update"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"title": "Reader2", "url": "https://example.com/ch2"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(update.status(), StatusCode::NOT_FOUND);

    let delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/inventory/web-readers/{missing_id}/delete"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delete.status(), StatusCode::OK);

    let add_location = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/inventory/web-readers/{missing_id}/locations/add"
                ))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "device_id": "dev-1",
                        "path_or_url": "https://example.com/reader",
                        "storage_type": "platform"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add_location.status(), StatusCode::OK);

    let remove_location = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/inventory/web-readers/{missing_id}/locations/{missing_id}/remove"
                ))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(remove_location.status(), StatusCode::OK);
}

#[tokio::test]
async fn system_handlers_return_expected_payloads_and_bypass_auth() {
    let openapi = r#"{"openapi":"3.0.0","paths":{"/api/v1/system/health":{}}}"#;
    let app = app_with_openapi(openapi);

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/system/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(health.status(), StatusCode::OK);
    let health_body = to_bytes(health.into_body(), 1024)
        .await
        .expect("health bytes");
    let health_json: Value = serde_json::from_slice(&health_body).expect("health json");
    assert_eq!(health_json, json!({"status": "ok"}));

    let openapi_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/system/openapi")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(openapi_response.status(), StatusCode::OK);
    let openapi_body = to_bytes(openapi_response.into_body(), 64 * 1024)
        .await
        .expect("openapi bytes");
    let openapi_json: Value = serde_json::from_slice(&openapi_body).expect("openapi json");
    assert!(openapi_json["paths"].is_object());
    assert!(!openapi_json["paths"]
        .as_object()
        .expect("paths object")
        .is_empty());
}

#[tokio::test]
async fn chapter_check_handler_returns_404_for_unknown_resource() {
    let app = app_with_openapi("{}");
    let unknown_id = uuid::Uuid::new_v4();

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/inventory/web-readers/{unknown_id}/check"
                ))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn chapter_check_history_returns_empty_list() {
    let app = app_with_openapi("{}");
    let id = uuid::Uuid::new_v4();

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/web-readers/{id}/checks"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.expect("bytes");
    let list: Value = serde_json::from_slice(&body).expect("json");
    assert!(list.as_array().expect("array").is_empty());
}

#[tokio::test]
async fn notifications_list_returns_empty() {
    let app = app_with_openapi("{}");

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/notifications")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.expect("bytes");
    let list: Value = serde_json::from_slice(&body).expect("json");
    assert!(list.as_array().expect("array").is_empty());
}

#[tokio::test]
async fn batch_import_partial_success_with_invalid_entry() {
    let app = app_with_openapi("{}");

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/ebooks/batch-import")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!([
                        {"title": "Book A"},
                        {"title": ""},
                        {"title": "Book C"}
                    ])
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.expect("bytes");
    let result: Value = serde_json::from_slice(&body).expect("json");
    let succeeded = result["succeeded"].as_array().expect("succeeded");
    let failed = result["failed"].as_array().expect("failed");
    assert_eq!(succeeded.len(), 2, "two valid entries should succeed");
    assert_eq!(failed.len(), 1, "one invalid entry should fail");
    assert_eq!(failed[0]["index"], 1);
}

#[tokio::test]
async fn image_handlers_cover_crud_and_location_routes() {
    let app = app_with_openapi("{}");
    let missing_id = "550e8400-e29b-41d4-a716-446655440000";

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/images/list")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list.status(), StatusCode::OK);

    let search = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/images/search?q=photo")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(search.status(), StatusCode::OK);

    let add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/images/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Photo",
                        "file_format": "png",
                        "width": 1920,
                        "height": 1080
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add.status(), StatusCode::OK);

    let invalid_add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/images/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"title": "Photo", "file_format": "exe"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(invalid_add.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/images/{missing_id}/detail"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(detail.status(), StatusCode::NOT_FOUND);

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/inventory/images/{missing_id}/update"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"title": "Updated"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(update.status(), StatusCode::NOT_FOUND);

    let delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/inventory/images/{missing_id}/delete"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delete.status(), StatusCode::OK);

    let add_location = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/inventory/images/{missing_id}/locations/add"
                ))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "device_id": "dev-1",
                        "path_or_url": "/photos/image.png",
                        "storage_type": "localfs"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add_location.status(), StatusCode::OK);

    let remove_location = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/inventory/images/{missing_id}/locations/{missing_id}/remove"
                ))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(remove_location.status(), StatusCode::OK);
}

#[tokio::test]
async fn video_handlers_cover_crud_and_location_routes() {
    let app = app_with_openapi("{}");
    let missing_id = "550e8400-e29b-41d4-a716-446655440000";

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/videos/list")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list.status(), StatusCode::OK);

    let search = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/videos/search?q=movie")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(search.status(), StatusCode::OK);

    let add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/videos/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Movie",
                        "file_format": "mkv",
                        "duration_secs": 7200,
                        "resolution": "1920x1080"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add.status(), StatusCode::OK);

    let invalid_add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/videos/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"title": "Movie", "file_format": "exe"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(invalid_add.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/videos/{missing_id}/detail"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(detail.status(), StatusCode::NOT_FOUND);

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/inventory/videos/{missing_id}/update"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"title": "Updated"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(update.status(), StatusCode::NOT_FOUND);

    let delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/inventory/videos/{missing_id}/delete"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delete.status(), StatusCode::OK);

    let add_location = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/inventory/videos/{missing_id}/locations/add"
                ))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "device_id": "dev-1",
                        "path_or_url": "/videos/movie.mkv",
                        "storage_type": "nas"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add_location.status(), StatusCode::OK);

    let remove_location = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/inventory/videos/{missing_id}/locations/{missing_id}/remove"
                ))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(remove_location.status(), StatusCode::OK);
}

#[tokio::test]
async fn game_handlers_cover_crud_and_location_routes() {
    let app = app_with_openapi("{}");
    let missing_id = "550e8400-e29b-41d4-a716-446655440000";

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/games/list")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list.status(), StatusCode::OK);

    let search = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/inventory/games/search?q=zelda")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(search.status(), StatusCode::OK);

    let add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/games/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Zelda",
                        "platform": "Switch",
                        "store": "eShop"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add.status(), StatusCode::OK);

    let invalid_add = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/games/add")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"title": ""}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(invalid_add.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/games/{missing_id}/detail"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(detail.status(), StatusCode::NOT_FOUND);

    let update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/inventory/games/{missing_id}/update"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"title": "Updated"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(update.status(), StatusCode::NOT_FOUND);

    let delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/inventory/games/{missing_id}/delete"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delete.status(), StatusCode::OK);

    let add_location = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/inventory/games/{missing_id}/locations/add"
                ))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "device_id": "switch-1",
                        "path_or_url": "digital",
                        "storage_type": "platform"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(add_location.status(), StatusCode::OK);

    let remove_location = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/inventory/games/{missing_id}/locations/{missing_id}/remove"
                ))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(remove_location.status(), StatusCode::OK);
}

#[tokio::test]
async fn device_handlers_return_503_when_service_not_configured() {
    let openapi = r#"{"paths":{"/api/v1/system/health":{}}}"#;
    let app = app_with_openapi(openapi);

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/devices")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list.status(), StatusCode::SERVICE_UNAVAILABLE);

    let current = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/devices/current")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(current.status(), StatusCode::SERVICE_UNAVAILABLE);

    let register = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/register")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"device_id": "desktop-home"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(register.status(), StatusCode::SERVICE_UNAVAILABLE);

    let delink = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/desktop-home/delink")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delink.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn device_list_handler_returns_200_with_empty_array() {
    let app = app_with_device_service("current-dev");
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/devices")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_array(), "body must be a JSON array");
}

#[tokio::test]
async fn device_current_handler_returns_404_when_not_registered() {
    let app = app_with_device_service("unregistered-dev");
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/devices/current")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn device_current_handler_returns_200_when_registered() {
    let app = app_with_device_service_and_device("my-dev");
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/devices/current")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["device_id"], "my-dev");
    assert_eq!(json["is_current"], true);
}

#[tokio::test]
async fn device_register_handler_returns_201_with_device_info() {
    let app = app_with_device_service("current-dev");
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/register")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"device_id": "new-dev", "device_name": "New PC"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["device_id"], "new-dev");
    assert_eq!(json["device_name"], "New PC");
}

#[tokio::test]
async fn device_register_handler_rejects_empty_device_id_with_422() {
    let app = app_with_device_service("current-dev");
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/register")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"device_id": ""}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn device_delink_handler_returns_422_for_current_device() {
    let app = app_with_device_service_and_device("self-dev");
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/self-dev/delink")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn device_delink_handler_returns_404_for_unknown_device() {
    let app = app_with_device_service("current-dev");
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/ghost-dev/delink")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn device_delink_handler_returns_200_for_other_device() {
    let app = app_with_device_service_and_device("current-dev");
    // Register a second device to delink
    let register_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/register")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"device_id": "other-dev"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("register response");
    assert_eq!(register_resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/other-dev/delink")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── In-memory ProgressRepository for handler tests ────────────────────────

struct FakeHandlerProgressRepo {
    storage: std::sync::Mutex<HashMap<String, ResourceProgress>>,
}

impl FakeHandlerProgressRepo {
    fn empty() -> Arc<Self> {
        Arc::new(Self {
            storage: std::sync::Mutex::new(HashMap::new()),
        })
    }

    fn with_progress(resource_id: &str, progress: f64, notes: Option<&str>) -> Arc<Self> {
        let repo = Self::empty();
        repo.storage.lock().unwrap().insert(
            resource_id.to_string(),
            ResourceProgress {
                resource_id: resource_id.to_string(),
                progress,
                notes: notes.map(str::to_string),
                updated_at: Utc::now(),
            },
        );
        repo
    }
}

#[async_trait]
impl ProgressRepository for FakeHandlerProgressRepo {
    async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError> {
        Ok(self.storage.lock().unwrap().get(resource_id).cloned())
    }

    async fn upsert(
        &self,
        resource_id: &str,
        progress: f64,
        notes: Option<&str>,
    ) -> Result<ResourceProgress, DomainError> {
        let record = ResourceProgress {
            resource_id: resource_id.to_string(),
            progress,
            notes: notes.map(str::to_string),
            updated_at: Utc::now(),
        };
        self.storage
            .lock()
            .unwrap()
            .insert(resource_id.to_string(), record.clone());
        Ok(record)
    }
}

// ── Fake repos for type-check tests ──────────────────────────────────────

struct FakeHandlerResourceRepo {
    resources: std::sync::Mutex<Vec<domain::Resource>>,
}

impl FakeHandlerResourceRepo {
    fn with_resource(id: Uuid, resource_type: ResourceType) -> Arc<Self> {
        let now = Utc::now();
        let resource = domain::Resource {
            id,
            title: "Test Resource".to_string(),
            notes: None,
            resource_type,
            created_at: now,
            updated_at: now,
        };
        Arc::new(Self {
            resources: std::sync::Mutex::new(vec![resource]),
        })
    }

    fn with_resources(resources: Vec<domain::Resource>) -> Arc<Self> {
        Arc::new(Self {
            resources: std::sync::Mutex::new(resources),
        })
    }
}

#[async_trait]
impl ResourceRepository for FakeHandlerResourceRepo {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        Ok(self.resources.lock().unwrap().clone())
    }
    async fn search(&self, _query: &str) -> Result<Vec<Resource>, DomainError> {
        Ok(vec![])
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
        self.resources
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or_else(|| DomainError::NotFound(format!("resource {id} not found")))
    }
    async fn create(&self, input: domain::NewResource) -> Result<Resource, DomainError> {
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
    async fn update(&self, id: Uuid, _input: domain::UpdateResource) -> Result<Resource, DomainError> {
        Err(DomainError::NotFound(format!("resource {id} not found")))
    }
    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }
}

struct FakeHandlerEbookMetaRepo {
    seeded_id: Option<Uuid>,
}

impl FakeHandlerEbookMetaRepo {
    fn for_id(id: Uuid) -> Arc<Self> {
        Arc::new(Self { seeded_id: Some(id) })
    }
    fn empty() -> Arc<Self> {
        Arc::new(Self { seeded_id: None })
    }
}

#[async_trait]
impl EbookMetaRepository for FakeHandlerEbookMetaRepo {
    async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        if self.seeded_id == Some(resource_id) {
            Ok(EbookMeta {
                resource_id,
                author: None,
                isbn: None,
                publisher: None,
                language: None,
                file_format: None,
            })
        } else {
            Err(DomainError::NotFound(format!("ebook meta for {resource_id} not found")))
        }
    }
    async fn upsert(&self, resource_id: Uuid, input: NewEbookMeta) -> Result<EbookMeta, DomainError> {
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

struct FakeHandlerWebReaderMetaRepo {
    seeded_id: Option<Uuid>,
}

impl FakeHandlerWebReaderMetaRepo {
    fn for_id(id: Uuid) -> Arc<Self> {
        Arc::new(Self { seeded_id: Some(id) })
    }
    fn empty() -> Arc<Self> {
        Arc::new(Self { seeded_id: None })
    }
}

#[async_trait]
impl WebReaderMetaRepository for FakeHandlerWebReaderMetaRepo {
    async fn get(&self, resource_id: Uuid) -> Result<WebReaderMeta, DomainError> {
        if self.seeded_id == Some(resource_id) {
            Ok(WebReaderMeta {
                resource_id,
                url: "https://example.com".to_string(),
                site_name: None,
                last_checked_chapter: None,
                check_interval_secs: None,
                last_checked_at: None,
                progress_css_selector: None,
            })
        } else {
            Err(DomainError::NotFound(format!("web reader meta for {resource_id} not found")))
        }
    }
    async fn upsert(&self, resource_id: Uuid, input: NewWebReaderMeta) -> Result<WebReaderMeta, DomainError> {
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

struct FakeHandlerImageMetaRepo {
    seeded_id: Option<Uuid>,
}

impl FakeHandlerImageMetaRepo {
    fn for_id(id: Uuid) -> Arc<Self> {
        Arc::new(Self { seeded_id: Some(id) })
    }
    fn empty() -> Arc<Self> {
        Arc::new(Self { seeded_id: None })
    }
}

#[async_trait]
impl ImageMetaRepository for FakeHandlerImageMetaRepo {
    async fn get(&self, resource_id: Uuid) -> Result<ImageMeta, DomainError> {
        if self.seeded_id == Some(resource_id) {
            Ok(ImageMeta {
                resource_id,
                width: None,
                height: None,
                file_format: None,
                file_size_bytes: None,
            })
        } else {
            Err(DomainError::NotFound(format!("image meta for {resource_id} not found")))
        }
    }
    async fn upsert(&self, resource_id: Uuid, input: NewImageMeta) -> Result<ImageMeta, DomainError> {
        Ok(ImageMeta {
            resource_id,
            width: input.width,
            height: input.height,
            file_format: input.file_format,
            file_size_bytes: input.file_size_bytes,
        })
    }
}

struct FakeHandlerVideoMetaRepo {
    seeded_id: Option<Uuid>,
}

impl FakeHandlerVideoMetaRepo {
    fn for_id(id: Uuid) -> Arc<Self> {
        Arc::new(Self { seeded_id: Some(id) })
    }
    fn empty() -> Arc<Self> {
        Arc::new(Self { seeded_id: None })
    }
}

#[async_trait]
impl VideoMetaRepository for FakeHandlerVideoMetaRepo {
    async fn get(&self, resource_id: Uuid) -> Result<VideoMeta, DomainError> {
        if self.seeded_id == Some(resource_id) {
            Ok(VideoMeta {
                resource_id,
                duration_secs: None,
                file_format: None,
                resolution: None,
                file_size_bytes: None,
            })
        } else {
            Err(DomainError::NotFound(format!("video meta for {resource_id} not found")))
        }
    }
    async fn upsert(&self, resource_id: Uuid, input: NewVideoMeta) -> Result<VideoMeta, DomainError> {
        Ok(VideoMeta {
            resource_id,
            duration_secs: input.duration_secs,
            file_format: input.file_format,
            resolution: input.resolution,
            file_size_bytes: input.file_size_bytes,
        })
    }
}

struct FakeHandlerGameMetaRepo {
    seeded_id: Option<Uuid>,
}

impl FakeHandlerGameMetaRepo {
    fn for_id(id: Uuid) -> Arc<Self> {
        Arc::new(Self { seeded_id: Some(id) })
    }
    fn empty() -> Arc<Self> {
        Arc::new(Self { seeded_id: None })
    }
}

#[async_trait]
impl GameMetaRepository for FakeHandlerGameMetaRepo {
    async fn get(&self, resource_id: Uuid) -> Result<GameMeta, DomainError> {
        if self.seeded_id == Some(resource_id) {
            Ok(GameMeta {
                resource_id,
                platform: None,
                store: None,
                developer: None,
                publisher: None,
                manual_notes: None,
            })
        } else {
            Err(DomainError::NotFound(format!("game meta for {resource_id} not found")))
        }
    }
    async fn upsert(&self, resource_id: Uuid, input: NewGameMeta) -> Result<GameMeta, DomainError> {
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

struct FakeHandlerLocationRepo;

#[async_trait]
impl LocationRepository for FakeHandlerLocationRepo {
    async fn list(&self, _resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
        Ok(vec![])
    }
    async fn add(&self, resource_id: Uuid, input: NewResourceLocation) -> Result<ResourceLocation, DomainError> {
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

fn app_with_progress_service_for_resource(
    resource_type: ResourceType,
    id: Uuid,
    progress_repo: Arc<FakeHandlerProgressRepo>,
) -> axum::Router {
    let openapi = r#"{"paths":{"/api/v1/system/health":{}}}"#;
    let resource_repo = FakeHandlerResourceRepo::with_resource(id, resource_type.clone());
    let location_repo = Arc::new(FakeHandlerLocationRepo);

    let ebook_meta = match resource_type {
        ResourceType::Ebook => FakeHandlerEbookMetaRepo::for_id(id),
        _ => FakeHandlerEbookMetaRepo::empty(),
    };
    let web_reader_meta = match resource_type {
        ResourceType::WebReader => FakeHandlerWebReaderMetaRepo::for_id(id),
        _ => FakeHandlerWebReaderMetaRepo::empty(),
    };
    let image_meta = match resource_type {
        ResourceType::Image => FakeHandlerImageMetaRepo::for_id(id),
        _ => FakeHandlerImageMetaRepo::empty(),
    };
    let video_meta = match resource_type {
        ResourceType::Video => FakeHandlerVideoMetaRepo::for_id(id),
        _ => FakeHandlerVideoMetaRepo::empty(),
    };
    let game_meta = match resource_type {
        ResourceType::Game => FakeHandlerGameMetaRepo::for_id(id),
        _ => FakeHandlerGameMetaRepo::empty(),
    };

    let ebook_service = Arc::new(EbookService::new(
        resource_repo.clone(),
        ebook_meta,
        location_repo.clone(),
    ));
    let web_reader_service = Arc::new(WebReaderService::new(
        resource_repo.clone(),
        web_reader_meta,
        location_repo.clone(),
    ));
    let image_service = Arc::new(ImageService::new(
        resource_repo.clone(),
        image_meta,
        location_repo.clone(),
    ));
    let video_service = Arc::new(VideoService::new(
        resource_repo.clone(),
        video_meta,
        location_repo.clone(),
    ));
    let game_service = Arc::new(GameService::new(
        resource_repo,
        game_meta,
        location_repo,
    ));

    let progress_svc = Arc::new(services::ProgressService::new(
        progress_repo as Arc<dyn domain::progress::ProgressRepository>,
    ));

    let mut state = AppState::new(
        ebook_service,
        web_reader_service,
        image_service,
        video_service,
        game_service,
        "secret-key".to_string(),
    );
    state.openapi_json = openapi.to_string();
    let state = state.with_progress_service(progress_svc);
    build_router(Arc::new(state))
}

#[tokio::test]
async fn handler_get_progress_returns_404_when_none() {
    let id = Uuid::new_v4();
    let app = app_with_progress_service_for_resource(
        ResourceType::Ebook,
        id,
        FakeHandlerProgressRepo::empty(),
    );

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/ebooks/{id}/progress"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn handler_get_progress_returns_200_when_set() {
    let id = Uuid::new_v4();
    let repo = FakeHandlerProgressRepo::with_progress(&id.to_string(), 0.25, Some("start"));
    let app = app_with_progress_service_for_resource(ResourceType::Ebook, id, repo);

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/ebooks/{id}/progress"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["resource_id"], id.to_string());
    assert!((json["progress"].as_f64().unwrap() - 0.25).abs() < f64::EPSILON);
    assert_eq!(json["notes"], "start");
    assert!(json["updated_at"].is_string());
}

#[tokio::test]
async fn handler_patch_progress_creates_and_returns_200() {
    let id = Uuid::new_v4();
    let repo = FakeHandlerProgressRepo::empty();
    let app = app_with_progress_service_for_resource(ResourceType::Ebook, id, repo.clone());

    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/inventory/ebooks/{id}/progress"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"progress": 0.75, "notes": "checkpoint"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!((json["progress"].as_f64().unwrap() - 0.75).abs() < f64::EPSILON);
    assert_eq!(json["notes"], "checkpoint");
    assert!(json["updated_at"].is_string());

    let stored = repo.storage.lock().unwrap();
    assert!(stored.contains_key(&id.to_string()), "repo must have saved the record");
}

#[tokio::test]
async fn handler_patch_progress_rejects_out_of_range() {
    let id = Uuid::new_v4();
    let app = app_with_progress_service_for_resource(
        ResourceType::Ebook,
        id,
        FakeHandlerProgressRepo::empty(),
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/inventory/ebooks/{id}/progress"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"progress": 1.5}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json["error"].as_str().unwrap_or("").contains("progress must be 0.0"),
        "error body must mention range: {:?}",
        json
    );
}

#[tokio::test]
async fn handler_get_progress_wrong_type_route_returns_404() {
    let id = Uuid::new_v4();
    // Seed as ebook + progress, but request via video route
    let repo = FakeHandlerProgressRepo::with_progress(&id.to_string(), 0.5, None);
    let app = app_with_progress_service_for_resource(ResourceType::Ebook, id, repo);

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/videos/{id}/progress"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn openapi_smoke_all_progress_get_paths_exist() {
    let json_str = adapters::generate_openapi_json();
    let doc: Value = serde_json::from_str(&json_str).expect("valid OpenAPI JSON");
    let paths = doc["paths"].as_object().expect("paths object");

    for kind in &["ebooks", "web-readers", "images", "videos", "games"] {
        let path = format!("/api/v1/inventory/{kind}/{{id}}/progress");
        assert!(
            paths.contains_key(&path),
            "expected path '{path}' in OpenAPI but not found; available: {:?}",
            paths.keys().collect::<Vec<_>>()
        );
    }
}

// ── Fake repos for tag handler tests ─────────────────────────────────────

struct FakeHandlerTagRepo {
    tags: std::sync::Mutex<Vec<Tag>>,
}

impl FakeHandlerTagRepo {
    fn empty() -> Arc<Self> {
        Arc::new(Self {
            tags: std::sync::Mutex::new(vec![]),
        })
    }

    fn with_tag(id: &str, name: &str) -> Arc<Self> {
        let repo = Self::empty();
        repo.tags.lock().unwrap().push(Tag {
            id: id.to_string(),
            name: name.to_string(),
            created_at: Utc::now(),
        });
        repo
    }
}

#[async_trait]
impl TagRepository for FakeHandlerTagRepo {
    async fn list(&self) -> Result<Vec<Tag>, DomainError> {
        Ok(self.tags.lock().unwrap().clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Tag>, DomainError> {
        Ok(self.tags.lock().unwrap().iter().find(|t| t.id == id).cloned())
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Tag>, DomainError> {
        Ok(self.tags.lock().unwrap().iter().find(|t| t.name == name).cloned())
    }

    async fn create(&self, name: &str) -> Result<Tag, DomainError> {
        if self
            .tags
            .lock()
            .unwrap()
            .iter()
            .any(|tag| tag.name == name)
        {
            return Err(DomainError::Conflict(format!(
                "tag '{name}' already exists"
            )));
        }
        let tag = Tag {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            created_at: Utc::now(),
        };
        self.tags.lock().unwrap().push(tag.clone());
        Ok(tag)
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut tags = self.tags.lock().unwrap();
        let pos = tags
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| DomainError::NotFound(format!("tag {id} not found")))?;
        tags.remove(pos);
        Ok(())
    }
}

struct FakeHandlerResourceTagRepo {
    associations: std::sync::Mutex<Vec<(String, String)>>,
    tag_repo: Arc<FakeHandlerTagRepo>,
}

impl FakeHandlerResourceTagRepo {
    fn empty(tag_repo: Arc<FakeHandlerTagRepo>) -> Arc<Self> {
        Arc::new(Self {
            associations: std::sync::Mutex::new(vec![]),
            tag_repo,
        })
    }

    fn with_association(
        tag_repo: Arc<FakeHandlerTagRepo>,
        resource_id: &str,
        tag_id: &str,
    ) -> Arc<Self> {
        let repo = Self::empty(tag_repo);
        repo.associations
            .lock()
            .unwrap()
            .push((resource_id.to_string(), tag_id.to_string()));
        repo
    }
}

#[async_trait]
impl ResourceTagRepository for FakeHandlerResourceTagRepo {
    async fn tags_for_resource(&self, resource_id: &str) -> Result<Vec<Tag>, DomainError> {
        let tag_ids: Vec<String> = self
            .associations
            .lock()
            .unwrap()
            .iter()
            .filter(|(rid, _)| rid == resource_id)
            .map(|(_, tid)| tid.clone())
            .collect();
        let tags = self.tag_repo.tags.lock().unwrap();
        Ok(tags.iter().filter(|t| tag_ids.contains(&t.id)).cloned().collect())
    }

    async fn attach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        if self
            .tag_repo
            .tags
            .lock()
            .unwrap()
            .iter()
            .all(|tag| tag.id != tag_id)
        {
            return Err(DomainError::NotFound(format!("tag {tag_id} not found")));
        }
        let mut assocs = self.associations.lock().unwrap();
        if !assocs.iter().any(|(rid, tid)| rid == resource_id && tid == tag_id) {
            assocs.push((resource_id.to_string(), tag_id.to_string()));
        }
        Ok(())
    }

    async fn detach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError> {
        let mut assocs = self.associations.lock().unwrap();
        let pos = assocs
            .iter()
            .position(|(rid, tid)| rid == resource_id && tid == tag_id)
            .ok_or_else(|| DomainError::NotFound("association not found".into()))?;
        assocs.remove(pos);
        Ok(())
    }

    async fn resource_ids_with_tag_id(&self, tag_id: &str) -> Result<Vec<String>, DomainError> {
        Ok(self
            .associations
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, tid)| tid == tag_id)
            .map(|(rid, _)| rid.clone())
            .collect())
    }
}

fn app_with_tag_service_for_resource(
    resource_type: ResourceType,
    id: Uuid,
    tag_repo: Arc<FakeHandlerTagRepo>,
    resource_tag_repo: Arc<FakeHandlerResourceTagRepo>,
) -> axum::Router {
    let openapi = r#"{"paths":{"/api/v1/system/health":{}}}"#;
    let resource_repo = FakeHandlerResourceRepo::with_resource(id, resource_type.clone());
    let location_repo = Arc::new(FakeHandlerLocationRepo);

    let ebook_meta = match resource_type {
        ResourceType::Ebook => FakeHandlerEbookMetaRepo::for_id(id),
        _ => FakeHandlerEbookMetaRepo::empty(),
    };
    let web_reader_meta = match resource_type {
        ResourceType::WebReader => FakeHandlerWebReaderMetaRepo::for_id(id),
        _ => FakeHandlerWebReaderMetaRepo::empty(),
    };
    let image_meta = match resource_type {
        ResourceType::Image => FakeHandlerImageMetaRepo::for_id(id),
        _ => FakeHandlerImageMetaRepo::empty(),
    };
    let video_meta = match resource_type {
        ResourceType::Video => FakeHandlerVideoMetaRepo::for_id(id),
        _ => FakeHandlerVideoMetaRepo::empty(),
    };
    let game_meta = match resource_type {
        ResourceType::Game => FakeHandlerGameMetaRepo::for_id(id),
        _ => FakeHandlerGameMetaRepo::empty(),
    };

    let ebook_service = Arc::new(EbookService::new(
        resource_repo.clone(),
        ebook_meta,
        location_repo.clone(),
    ));
    let web_reader_service = Arc::new(WebReaderService::new(
        resource_repo.clone(),
        web_reader_meta,
        location_repo.clone(),
    ));
    let image_service = Arc::new(ImageService::new(
        resource_repo.clone(),
        image_meta,
        location_repo.clone(),
    ));
    let video_service = Arc::new(VideoService::new(
        resource_repo.clone(),
        video_meta,
        location_repo.clone(),
    ));
    let game_service = Arc::new(GameService::new(resource_repo, game_meta, location_repo));

    let tr: Arc<dyn TagRepository> = tag_repo;
    let rtr: Arc<dyn ResourceTagRepository> = resource_tag_repo;
    let tag_svc = Arc::new(TagService::new(tr, rtr));

    let mut state = AppState::new(
        ebook_service,
        web_reader_service,
        image_service,
        video_service,
        game_service,
        "secret-key".to_string(),
    );
    state.openapi_json = openapi.to_string();
    let state = state.with_tag_service(tag_svc);
    build_router(Arc::new(state))
}

fn app_with_tag_service_for_list_resources(
    resources: Vec<(Uuid, ResourceType, &str)>,
    tag_repo: Arc<FakeHandlerTagRepo>,
    resource_tag_repo: Arc<FakeHandlerResourceTagRepo>,
) -> axum::Router {
    let openapi = r#"{"paths":{"/api/v1/system/health":{}}}"#;
    let now = Utc::now();
    let resource_repo = FakeHandlerResourceRepo::with_resources(
        resources
            .into_iter()
            .map(|(id, resource_type, title)| domain::Resource {
                id,
                title: title.to_string(),
                notes: None,
                resource_type,
                created_at: now,
                updated_at: now,
            })
            .collect(),
    );
    let location_repo = Arc::new(FakeHandlerLocationRepo);

    let ebook_service = Arc::new(EbookService::new(
        resource_repo.clone(),
        FakeHandlerEbookMetaRepo::empty(),
        location_repo.clone(),
    ));
    let web_reader_service = Arc::new(WebReaderService::new(
        resource_repo.clone(),
        FakeHandlerWebReaderMetaRepo::empty(),
        location_repo.clone(),
    ));
    let image_service = Arc::new(ImageService::new(
        resource_repo.clone(),
        FakeHandlerImageMetaRepo::empty(),
        location_repo.clone(),
    ));
    let video_service = Arc::new(VideoService::new(
        resource_repo.clone(),
        FakeHandlerVideoMetaRepo::empty(),
        location_repo.clone(),
    ));
    let game_service = Arc::new(GameService::new(
        resource_repo,
        FakeHandlerGameMetaRepo::empty(),
        location_repo,
    ));

    let tr: Arc<dyn TagRepository> = tag_repo;
    let rtr: Arc<dyn ResourceTagRepository> = resource_tag_repo;
    let tag_svc = Arc::new(TagService::new(tr, rtr));

    let mut state = AppState::new(
        ebook_service,
        web_reader_service,
        image_service,
        video_service,
        game_service,
        "secret-key".to_string(),
    );
    state.openapi_json = openapi.to_string();
    let state = state.with_tag_service(tag_svc);
    build_router(Arc::new(state))
}

async fn assert_list_tag_filter_and_empty_ignore(route_prefix: &str, resource_type: ResourceType) {
    let tagged_id = Uuid::new_v4();
    let untagged_id = Uuid::new_v4();
    let tag_id = Uuid::new_v4().to_string();
    let tag_repo = FakeHandlerTagRepo::with_tag(&tag_id, "sci-fi");
    let resource_tag_repo = FakeHandlerResourceTagRepo::with_association(
        tag_repo.clone(),
        &tagged_id.to_string(),
        &tag_id,
    );

    let app = app_with_tag_service_for_list_resources(
        vec![
            (tagged_id, resource_type.clone(), "Tagged"),
            (untagged_id, resource_type, "Untagged"),
        ],
        tag_repo,
        resource_tag_repo,
    );

    let filtered = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/{route_prefix}/list?tag=sci-fi"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(filtered.status(), StatusCode::OK);
    let filtered_body = to_bytes(filtered.into_body(), usize::MAX).await.unwrap();
    let filtered_json: Value = serde_json::from_slice(&filtered_body).unwrap();
    let filtered_items = filtered_json.as_array().expect("expected array");
    assert_eq!(filtered_items.len(), 1);
    assert_eq!(filtered_items[0]["id"], tagged_id.to_string());

    let ignored = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/{route_prefix}/list?tag="))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(ignored.status(), StatusCode::OK);
    let ignored_body = to_bytes(ignored.into_body(), usize::MAX).await.unwrap();
    let ignored_json: Value = serde_json::from_slice(&ignored_body).unwrap();
    let ignored_items = ignored_json.as_array().expect("expected array");
    assert_eq!(ignored_items.len(), 2);
}

// ── Tag handler tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn handler_list_tags_returns_empty_list() {
    let tag_repo = FakeHandlerTagRepo::empty();
    let resource_tag_repo = FakeHandlerResourceTagRepo::empty(tag_repo.clone());
    let id = Uuid::new_v4();
    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tags")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, json!([]));
}

#[tokio::test]
async fn handler_create_tag_returns_201() {
    let tag_repo = FakeHandlerTagRepo::empty();
    let resource_tag_repo = FakeHandlerResourceTagRepo::empty(tag_repo.clone());
    let id = Uuid::new_v4();
    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tags")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "  Sci-Fi  "}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "sci-fi");
    assert!(json["id"].is_string());
    assert!(json["created_at"].is_string());
}

#[tokio::test]
async fn handler_create_tag_duplicate_name_returns_409() {
    let tag_id = Uuid::new_v4().to_string();
    let tag_repo = FakeHandlerTagRepo::with_tag(&tag_id, "sci-fi");
    let resource_tag_repo = FakeHandlerResourceTagRepo::empty(tag_repo.clone());
    let id = Uuid::new_v4();
    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tags")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "  Sci-Fi  "}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn handler_delete_tag_returns_204() {
    let tag_id = Uuid::new_v4().to_string();
    let tag_repo = FakeHandlerTagRepo::with_tag(&tag_id, "fiction");
    let resource_tag_repo = FakeHandlerResourceTagRepo::empty(tag_repo.clone());
    let id = Uuid::new_v4();
    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo.clone(),
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/tags/{tag_id}"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert!(
        tag_repo.tags.lock().unwrap().is_empty(),
        "tag must be removed from repo"
    );
}

#[tokio::test]
async fn handler_get_resource_tags_returns_list() {
    let id = Uuid::new_v4();
    let tag_id_1 = Uuid::new_v4().to_string();
    let tag_id_2 = Uuid::new_v4().to_string();

    let tag_repo = Arc::new(FakeHandlerTagRepo {
        tags: std::sync::Mutex::new(vec![
            Tag {
                id: tag_id_1.clone(),
                name: "fantasy".to_string(),
                created_at: Utc::now(),
            },
            Tag {
                id: tag_id_2.clone(),
                name: "sci-fi".to_string(),
                created_at: Utc::now(),
            },
        ]),
    });
    let resource_tag_repo = Arc::new(FakeHandlerResourceTagRepo {
        associations: std::sync::Mutex::new(vec![
            (id.to_string(), tag_id_1.clone()),
            (id.to_string(), tag_id_2.clone()),
        ]),
        tag_repo: tag_repo.clone(),
    });

    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/ebooks/{id}/tags"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().expect("expected array");
    assert_eq!(arr.len(), 2);
    let names: Vec<&str> = arr.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"fantasy"));
    assert!(names.contains(&"sci-fi"));
}

#[tokio::test]
async fn handler_attach_tag_to_resource_returns_200() {
    let id = Uuid::new_v4();
    let tag_id = Uuid::new_v4().to_string();
    let tag_repo = FakeHandlerTagRepo::with_tag(&tag_id, "action");
    let resource_tag_repo = FakeHandlerResourceTagRepo::empty(tag_repo.clone());
    let resource_tag_repo_ref = resource_tag_repo.clone();

    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/inventory/ebooks/{id}/tags"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"tag_id": tag_id}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let assocs = resource_tag_repo_ref.associations.lock().unwrap();
    assert!(
        assocs
            .iter()
            .any(|(rid, tid)| rid == &id.to_string() && tid == &tag_id),
        "association must be stored"
    );
}

#[tokio::test]
async fn handler_attach_tag_to_resource_missing_tag_returns_404() {
    let id = Uuid::new_v4();
    let missing_tag_id = Uuid::new_v4().to_string();
    let tag_repo = FakeHandlerTagRepo::empty();
    let resource_tag_repo = FakeHandlerResourceTagRepo::empty(tag_repo.clone());

    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/inventory/ebooks/{id}/tags"))
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(json!({"tag_id": missing_tag_id}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn handler_detach_tag_from_resource_returns_204() {
    let id = Uuid::new_v4();
    let tag_id = Uuid::new_v4().to_string();
    let tag_repo = FakeHandlerTagRepo::with_tag(&tag_id, "horror");
    let resource_tag_repo = FakeHandlerResourceTagRepo::with_association(
        tag_repo.clone(),
        &id.to_string(),
        &tag_id,
    );
    let resource_tag_repo_ref = resource_tag_repo.clone();

    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/inventory/ebooks/{id}/tags/{tag_id}"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    let assocs = resource_tag_repo_ref.associations.lock().unwrap();
    assert!(
        !assocs
            .iter()
            .any(|(rid, tid)| rid == &id.to_string() && tid == &tag_id),
        "association must be removed"
    );
}

#[tokio::test]
async fn handler_get_resource_tags_wrong_type_route_returns_404() {
    let id = Uuid::new_v4();
    let tag_id = Uuid::new_v4().to_string();
    let tag_repo = FakeHandlerTagRepo::with_tag(&tag_id, "drama");
    let resource_tag_repo = FakeHandlerResourceTagRepo::with_association(
        tag_repo.clone(),
        &id.to_string(),
        &tag_id,
    );

    // Seed resource as Ebook; access via videos route → should 404
    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/videos/{id}/tags"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn handler_get_resource_tags_unsupported_type_returns_422() {
    let id = Uuid::new_v4();
    let tag_repo = FakeHandlerTagRepo::empty();
    let resource_tag_repo = FakeHandlerResourceTagRepo::empty(tag_repo.clone());

    let app = app_with_tag_service_for_resource(
        ResourceType::Ebook,
        id,
        tag_repo,
        resource_tag_repo,
    );

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/inventory/toasters/{id}/tags"))
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn openapi_smoke_all_tag_paths_exist() {
    let json_str = adapters::generate_openapi_json();
    let doc: Value = serde_json::from_str(&json_str).expect("valid OpenAPI JSON");
    let paths = doc["paths"].as_object().expect("paths object");

    for path in &[
        "/api/v1/tags",
        "/api/v1/tags/{id}",
        "/api/v1/inventory/{type}/{id}/tags",
        "/api/v1/inventory/{type}/{id}/tags/{tag_id}",
    ] {
        assert!(
            paths.contains_key(*path),
            "expected path '{path}' in OpenAPI but not found; available: {:?}",
            paths.keys().collect::<Vec<_>>()
        );
    }
}

#[tokio::test]
async fn handler_list_ebooks_supports_tag_filter_and_ignores_empty_tag() {
    assert_list_tag_filter_and_empty_ignore("ebooks", ResourceType::Ebook).await;
}

#[tokio::test]
async fn handler_list_web_readers_supports_tag_filter_and_ignores_empty_tag() {
    assert_list_tag_filter_and_empty_ignore("web-readers", ResourceType::WebReader).await;
}

#[tokio::test]
async fn handler_list_images_supports_tag_filter_and_ignores_empty_tag() {
    assert_list_tag_filter_and_empty_ignore("images", ResourceType::Image).await;
}

#[tokio::test]
async fn handler_list_videos_supports_tag_filter_and_ignores_empty_tag() {
    assert_list_tag_filter_and_empty_ignore("videos", ResourceType::Video).await;
}

#[tokio::test]
async fn handler_list_games_supports_tag_filter_and_ignores_empty_tag() {
    assert_list_tag_filter_and_empty_ignore("games", ResourceType::Game).await;
}

// ── Batch-update handler tests (P7-J) ─────────────────────────────────────

// Resource repo that returns success on update (needed for batch-update tests).
struct FakeUpdatableResourceRepo {
    resources: std::sync::Mutex<Vec<Resource>>,
}

impl FakeUpdatableResourceRepo {
    fn with_resources(resources: Vec<Resource>) -> Arc<Self> {
        Arc::new(Self { resources: std::sync::Mutex::new(resources) })
    }
}

#[async_trait]
impl ResourceRepository for FakeUpdatableResourceRepo {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        Ok(self.resources.lock().unwrap().clone())
    }
    async fn search(&self, _query: &str) -> Result<Vec<Resource>, DomainError> {
        Ok(vec![])
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
        self.resources
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or_else(|| DomainError::NotFound(format!("resource {id} not found")))
    }
    async fn create(&self, input: domain::NewResource) -> Result<Resource, DomainError> {
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
    async fn update(&self, id: Uuid, input: domain::UpdateResource) -> Result<Resource, DomainError> {
        let mut resources = self.resources.lock().unwrap();
        let resource = resources
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| DomainError::NotFound(format!("resource {id} not found")))?;
        if let Some(title) = input.title {
            resource.title = title;
        }
        Ok(resource.clone())
    }
    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }
}

fn make_ebook_resource(id: Uuid) -> Resource {
    let now = Utc::now();
    Resource {
        id,
        title: "Test Ebook".to_string(),
        notes: None,
        resource_type: domain::ResourceType::Ebook,
        created_at: now,
        updated_at: now,
    }
}

struct FakeMultiIdEbookMetaRepo {
    ids: Vec<Uuid>,
}

impl FakeMultiIdEbookMetaRepo {
    fn new(ids: Vec<Uuid>) -> Self {
        Self { ids }
    }
}

#[async_trait]
impl EbookMetaRepository for FakeMultiIdEbookMetaRepo {
    async fn get(&self, resource_id: Uuid) -> Result<EbookMeta, DomainError> {
        if self.ids.contains(&resource_id) {
            Ok(EbookMeta {
                resource_id,
                author: None,
                isbn: None,
                publisher: None,
                language: None,
                file_format: None,
            })
        } else {
            Err(DomainError::NotFound(format!("ebook meta for {resource_id} not found")))
        }
    }
    async fn upsert(&self, resource_id: Uuid, input: NewEbookMeta) -> Result<EbookMeta, DomainError> {
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

fn app_for_batch_update_ebooks(ids: &[Uuid]) -> axum::Router {
    let resources: Vec<Resource> = ids.iter().map(|&id| make_ebook_resource(id)).collect();
    let resource_repo: Arc<dyn ResourceRepository> =
        FakeUpdatableResourceRepo::with_resources(resources);
    // Use an ebook meta repo that seeds all IDs via upsert (returns Ok for any ID)
    let ebook_meta = Arc::new(FakeMultiIdEbookMetaRepo::new(ids.to_vec()));
    let location_repo = Arc::new(FakeHandlerLocationRepo);
    let ebook_service = Arc::new(EbookService::new(
        resource_repo.clone(),
        ebook_meta,
        location_repo.clone(),
    ));
    let web_reader_service = Arc::new(WebReaderService::new(
        resource_repo.clone(),
        FakeHandlerWebReaderMetaRepo::empty(),
        location_repo.clone(),
    ));
    let image_service = Arc::new(ImageService::new(
        resource_repo.clone(),
        FakeHandlerImageMetaRepo::empty(),
        location_repo.clone(),
    ));
    let video_service = Arc::new(VideoService::new(
        resource_repo.clone(),
        FakeHandlerVideoMetaRepo::empty(),
        location_repo.clone(),
    ));
    let game_service = Arc::new(GameService::new(
        resource_repo,
        FakeHandlerGameMetaRepo::empty(),
        location_repo,
    ));
    let state = AppState::new(
        ebook_service,
        web_reader_service,
        image_service,
        video_service,
        game_service,
        "secret-key".to_string(),
    );
    build_router(Arc::new(state))
}

#[tokio::test]
async fn handler_batch_update_ebooks_returns_updated_count() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let app = app_for_batch_update_ebooks(&[id1, id2]);

    let body = json!({
        "ids": [id1.to_string(), id2.to_string()],
        "fields": { "author": "Batch Author" }
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/inventory/ebooks/batch-update")
                .header("content-type", "application/json")
                .header("authorization", "Bearer secret-key")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["updated"], 2);
    assert_eq!(json["failed"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn handler_batch_update_ebooks_reports_invalid_uuid_in_failed() {
    let id1 = Uuid::new_v4();
    let app = app_for_batch_update_ebooks(&[id1]);

    let body = json!({
        "ids": [id1.to_string(), "not-a-uuid"],
        "fields": { "author": "Batch Author" }
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/inventory/ebooks/batch-update")
                .header("content-type", "application/json")
                .header("authorization", "Bearer secret-key")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["updated"], 1);
    assert_eq!(json["failed"].as_array().unwrap().len(), 1);
    assert_eq!(json["failed"][0]["id"], "not-a-uuid");
}

// ── Batch copy-meta tests ─────────────────────────────────────────────────

fn app_for_batch_copy_ebook_meta(source_id: Uuid, target_ids: &[Uuid]) -> axum::Router {
    let mut all_ids = vec![source_id];
    all_ids.extend_from_slice(target_ids);
    let resources: Vec<Resource> = all_ids.iter().map(|&id| make_ebook_resource(id)).collect();
    let resource_repo: Arc<dyn ResourceRepository> =
        FakeUpdatableResourceRepo::with_resources(resources);
    let ebook_meta = Arc::new(FakeMultiIdEbookMetaRepo::new(all_ids));
    let location_repo = Arc::new(FakeHandlerLocationRepo);
    let ebook_service = Arc::new(EbookService::new(
        resource_repo.clone(),
        ebook_meta,
        location_repo.clone(),
    ));
    let web_reader_service = Arc::new(WebReaderService::new(
        resource_repo.clone(),
        FakeHandlerWebReaderMetaRepo::empty(),
        location_repo.clone(),
    ));
    let image_service = Arc::new(ImageService::new(
        resource_repo.clone(),
        FakeHandlerImageMetaRepo::empty(),
        location_repo.clone(),
    ));
    let video_service = Arc::new(VideoService::new(
        resource_repo.clone(),
        FakeHandlerVideoMetaRepo::empty(),
        location_repo.clone(),
    ));
    let game_service = Arc::new(GameService::new(
        resource_repo,
        FakeHandlerGameMetaRepo::empty(),
        location_repo,
    ));
    let state = AppState::new(
        ebook_service,
        web_reader_service,
        image_service,
        video_service,
        game_service,
        "secret-key".to_string(),
    );
    build_router(Arc::new(state))
}

#[tokio::test]
async fn handler_batch_copy_ebook_meta_returns_updated_count() {
    let source_id = Uuid::new_v4();
    let target1 = Uuid::new_v4();
    let target2 = Uuid::new_v4();
    let app = app_for_batch_copy_ebook_meta(source_id, &[target1, target2]);

    let body = json!({
        "source_id": source_id.to_string(),
        "target_ids": [target1.to_string(), target2.to_string()]
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/ebooks/batch-copy-meta")
                .header("content-type", "application/json")
                .header("authorization", "Bearer secret-key")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["updated"], 2);
    assert_eq!(json["failed"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn handler_batch_copy_ebook_meta_invalid_source_id_returns_400() {
    let target1 = Uuid::new_v4();
    let app = app_for_batch_copy_ebook_meta(Uuid::new_v4(), &[target1]);

    let body = json!({
        "source_id": "not-a-uuid",
        "target_ids": [target1.to_string()]
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/ebooks/batch-copy-meta")
                .header("content-type", "application/json")
                .header("authorization", "Bearer secret-key")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn handler_batch_copy_ebook_meta_unknown_source_reports_all_failed() {
    let source_id = Uuid::new_v4();
    let unknown_source = Uuid::new_v4();
    let target1 = Uuid::new_v4();
    // Only seed source_id in repo, not unknown_source
    let app = app_for_batch_copy_ebook_meta(source_id, &[target1]);

    let body = json!({
        "source_id": unknown_source.to_string(),
        "target_ids": [target1.to_string()]
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/ebooks/batch-copy-meta")
                .header("content-type", "application/json")
                .header("authorization", "Bearer secret-key")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["updated"], 0);
    assert_eq!(json["failed"].as_array().unwrap().len(), 1);
}
