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
    DomainError,
};
use serde_json::{json, Value};
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

fn app_with_progress_service(repo: Arc<FakeHandlerProgressRepo>) -> axum::Router {
    let openapi = r#"{"paths":{"/api/v1/system/health":{}}}"#;
    let svc = Arc::new(services::ProgressService::new(
        repo as Arc<dyn domain::progress::ProgressRepository>,
    ));
    let mut state = AppState::for_tests("secret-key".to_string());
    state.openapi_json = openapi.to_string();
    let state = state.with_progress_service(svc);
    build_router(Arc::new(state))
}

#[tokio::test]
async fn handler_get_progress_returns_404_when_none() {
    let id = Uuid::new_v4();
    let app = app_with_progress_service(FakeHandlerProgressRepo::empty());

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
    let app = app_with_progress_service(repo);

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
    let app = app_with_progress_service(repo.clone());

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
    let app = app_with_progress_service(FakeHandlerProgressRepo::empty());

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
