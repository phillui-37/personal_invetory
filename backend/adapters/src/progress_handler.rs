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

async fn get_progress_inner(
    id: Uuid,
    state: Arc<AppState>,
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

async fn patch_progress_inner(
    id: Uuid,
    body: PatchProgressRequest,
    state: Arc<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = progress_svc(&state)?;
    let record = svc
        .upsert(&id.to_string(), body.progress, body.notes.as_deref())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ProgressResponse::from(record)))
}

async fn check_ebook_exists(id: Uuid, state: &Arc<AppState>) -> Result<(), ApiError> {
    state
        .ebook_service
        .ebook_detail(id)
        .await
        .map(|_| ())
        .map_err(ApiError::from)
}

async fn check_web_reader_exists(id: Uuid, state: &Arc<AppState>) -> Result<(), ApiError> {
    state
        .web_reader_service
        .web_reader_detail(id)
        .await
        .map(|_| ())
        .map_err(ApiError::from)
}

async fn check_image_exists(id: Uuid, state: &Arc<AppState>) -> Result<(), ApiError> {
    state
        .image_service
        .image_detail(id)
        .await
        .map(|_| ())
        .map_err(ApiError::from)
}

async fn check_video_exists(id: Uuid, state: &Arc<AppState>) -> Result<(), ApiError> {
    state
        .video_service
        .video_detail(id)
        .await
        .map(|_| ())
        .map_err(ApiError::from)
}

async fn check_game_exists(id: Uuid, state: &Arc<AppState>) -> Result<(), ApiError> {
    state
        .game_service
        .game_detail(id)
        .await
        .map(|_| ())
        .map_err(ApiError::from)
}

/// Get reading/viewing progress for an ebook.
#[utoipa::path(
    get,
    path = "/api/v1/inventory/ebooks/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    responses(
        (status = 200, description = "Progress record", body = ProgressResponse),
        (status = 404, description = "No progress recorded yet or resource not found"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_ebook_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    check_ebook_exists(id, &state).await?;
    get_progress_inner(id, state).await
}

/// Upsert reading/viewing progress for an ebook.
#[utoipa::path(
    patch,
    path = "/api/v1/inventory/ebooks/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    request_body = PatchProgressRequest,
    responses(
        (status = 200, description = "Progress saved", body = ProgressResponse),
        (status = 404, description = "Resource not found"),
        (status = 422, description = "progress must be 0.0–1.0"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn patch_ebook_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PatchProgressRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_ebook_exists(id, &state).await?;
    patch_progress_inner(id, body, state).await
}

/// Get reading/viewing progress for a web reader.
#[utoipa::path(
    get,
    path = "/api/v1/inventory/web-readers/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    responses(
        (status = 200, description = "Progress record", body = ProgressResponse),
        (status = 404, description = "No progress recorded yet or resource not found"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_web_reader_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    check_web_reader_exists(id, &state).await?;
    get_progress_inner(id, state).await
}

/// Upsert reading/viewing progress for a web reader.
#[utoipa::path(
    patch,
    path = "/api/v1/inventory/web-readers/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    request_body = PatchProgressRequest,
    responses(
        (status = 200, description = "Progress saved", body = ProgressResponse),
        (status = 404, description = "Resource not found"),
        (status = 422, description = "progress must be 0.0–1.0"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn patch_web_reader_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PatchProgressRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_web_reader_exists(id, &state).await?;
    patch_progress_inner(id, body, state).await
}

/// Get reading/viewing progress for an image.
#[utoipa::path(
    get,
    path = "/api/v1/inventory/images/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    responses(
        (status = 200, description = "Progress record", body = ProgressResponse),
        (status = 404, description = "No progress recorded yet or resource not found"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_image_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    check_image_exists(id, &state).await?;
    get_progress_inner(id, state).await
}

/// Upsert reading/viewing progress for an image.
#[utoipa::path(
    patch,
    path = "/api/v1/inventory/images/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    request_body = PatchProgressRequest,
    responses(
        (status = 200, description = "Progress saved", body = ProgressResponse),
        (status = 404, description = "Resource not found"),
        (status = 422, description = "progress must be 0.0–1.0"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn patch_image_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PatchProgressRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_image_exists(id, &state).await?;
    patch_progress_inner(id, body, state).await
}

/// Get viewing progress for a video.
#[utoipa::path(
    get,
    path = "/api/v1/inventory/videos/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    responses(
        (status = 200, description = "Progress record", body = ProgressResponse),
        (status = 404, description = "No progress recorded yet or resource not found"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_video_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    check_video_exists(id, &state).await?;
    get_progress_inner(id, state).await
}

/// Upsert viewing progress for a video.
#[utoipa::path(
    patch,
    path = "/api/v1/inventory/videos/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    request_body = PatchProgressRequest,
    responses(
        (status = 200, description = "Progress saved", body = ProgressResponse),
        (status = 404, description = "Resource not found"),
        (status = 422, description = "progress must be 0.0–1.0"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn patch_video_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PatchProgressRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_video_exists(id, &state).await?;
    patch_progress_inner(id, body, state).await
}

/// Get playing progress for a game.
#[utoipa::path(
    get,
    path = "/api/v1/inventory/games/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    responses(
        (status = 200, description = "Progress record", body = ProgressResponse),
        (status = 404, description = "No progress recorded yet or resource not found"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_game_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    check_game_exists(id, &state).await?;
    get_progress_inner(id, state).await
}

/// Upsert playing progress for a game.
#[utoipa::path(
    patch,
    path = "/api/v1/inventory/games/{id}/progress",
    params(("id" = Uuid, Path, description = "Resource UUID")),
    request_body = PatchProgressRequest,
    responses(
        (status = 200, description = "Progress saved", body = ProgressResponse),
        (status = 404, description = "Resource not found"),
        (status = 422, description = "progress must be 0.0–1.0"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn patch_game_progress(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PatchProgressRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_game_exists(id, &state).await?;
    patch_progress_inner(id, body, state).await
}

