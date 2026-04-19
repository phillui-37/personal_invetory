use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use domain::DomainError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchProgressRequest {
    pub progress: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProgressResponse {
    pub resource_id: String,
    pub progress: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub updated_at: String,
}

impl From<domain::progress::ResourceProgress> for ProgressResponse {
    fn from(value: domain::progress::ResourceProgress) -> Self {
        Self {
            resource_id: value.resource_id,
            progress: value.progress,
            notes: value.notes,
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

fn progress_svc(state: &AppState) -> Result<&Arc<services::ProgressService>, ApiError> {
    state
        .progress_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("progress service not configured"))
}

/// Get reading/viewing progress for a resource.
#[utoipa::path(
    get,
    path = "/api/v1/inventory/ebooks/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    responses(
        (status = 200, description = "Progress record", body = ProgressResponse),
        (status = 404, description = "No progress recorded yet"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = progress_svc(&state)?;
    let result = svc.get(&id.to_string()).await.map_err(ApiError::from)?;
    match result {
        Some(p) => Ok(Json(ProgressResponse::from(p))),
        None => Err(ApiError::from(DomainError::NotFound(format!(
            "progress for resource {id} not found"
        )))),
    }
}

/// Upsert reading/viewing progress for a resource.
#[utoipa::path(
    patch,
    path = "/api/v1/inventory/ebooks/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    request_body = PatchProgressRequest,
    responses(
        (status = 200, description = "Progress saved", body = ProgressResponse),
        (status = 422, description = "progress must be 0.0–1.0"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn patch_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PatchProgressRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = progress_svc(&state)?;
    let record = svc
        .upsert(&id.to_string(), body.progress, body.notes.as_deref())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ProgressResponse::from(record)))
}
