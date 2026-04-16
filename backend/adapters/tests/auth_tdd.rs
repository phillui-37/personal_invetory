use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    response::IntoResponse,
    routing::get,
    Router,
};
use tower::ServiceExt;

use adapters::{auth_middleware, ApiError, AppState};
use domain::DomainError;

async fn protected_handler() -> impl IntoResponse {
    StatusCode::OK
}

#[tokio::test]
async fn auth_accepts_valid_api_key() {
    let state = Arc::new(AppState::for_tests("secret-key".to_string()));
    let app = Router::new()
        .route("/protected", get(protected_handler))
        .layer(from_fn_with_state(state, auth_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_rejects_missing_or_invalid_api_key() {
    let state = Arc::new(AppState::for_tests("secret-key".to_string()));
    let app = Router::new()
        .route("/protected", get(protected_handler))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    let missing = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

    let invalid = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("authorization", "Bearer wrong-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_rejects_empty_token_when_configured_key_is_empty() {
    let state = Arc::new(AppState::for_tests(String::new()));
    let app = Router::new()
        .route("/protected", get(protected_handler))
        .layer(from_fn_with_state(state, auth_middleware));

    let empty_token = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("authorization", "Bearer ")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(empty_token.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_bypasses_system_routes() {
    let state = Arc::new(AppState::for_tests("secret-key".to_string()));
    let app = Router::new()
        .route("/api/v1/system/health", get(protected_handler))
        .route("/api/v1/system/openapi", get(protected_handler))
        .layer(from_fn_with_state(state, auth_middleware));

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

    let openapi = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/system/openapi")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(openapi.status(), StatusCode::OK);
}

#[test]
fn api_error_maps_domain_error_to_expected_http_status() {
    let not_found = ApiError::from(DomainError::NotFound("x".to_string())).into_response();
    assert_eq!(not_found.status(), StatusCode::NOT_FOUND);

    let validation = ApiError::from(DomainError::ValidationError("x".to_string())).into_response();
    assert_eq!(validation.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let conflict = ApiError::from(DomainError::Conflict("x".to_string())).into_response();
    assert_eq!(conflict.status(), StatusCode::CONFLICT);

    let internal = ApiError::from(DomainError::InternalError("x".to_string())).into_response();
    assert_eq!(internal.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
