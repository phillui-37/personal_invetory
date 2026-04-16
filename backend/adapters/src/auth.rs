use std::sync::Arc;

use axum::{
    extract::Request,
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{ApiError, AppState};

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if path == "/api/v1/system/health" || path == "/api/v1/system/openapi" {
        return next.run(request).await;
    }

    let authorized = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| {
            let expected = state.api_key.trim();
            let provided = token.trim();
            !expected.is_empty() && !provided.is_empty() && provided == expected
        })
        .unwrap_or(false);

    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            ApiError::unauthorized("Invalid or missing API key"),
        )
            .into_response();
    }

    next.run(request).await
}
