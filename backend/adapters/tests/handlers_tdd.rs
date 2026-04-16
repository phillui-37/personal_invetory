use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use adapters::{build_router, AppState};

fn app_with_openapi(openapi_json: &str) -> axum::Router {
    let mut state = AppState::for_tests("secret-key".to_string());
    state.openapi_json = openapi_json.to_string();
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
