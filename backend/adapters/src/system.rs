use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::Value;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn openapi(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, ApiError> {
    let parsed: Value = serde_json::from_str(&state.openapi_json).map_err(|error| {
        ApiError::from(domain::DomainError::InternalError(format!(
            "invalid openapi json: {error}"
        )))
    })?;

    Ok((StatusCode::OK, Json(parsed)))
}
