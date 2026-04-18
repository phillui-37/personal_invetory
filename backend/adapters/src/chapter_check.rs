use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use domain::ChapterCheck;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[utoipa::path(
    post,
    path = "/api/v1/inventory/web-readers/{id}/check",
    params(("id" = Uuid, Path, description = "Web reader resource ID")),
    responses(
        (status = 200, description = "Chapter check result", body = ChapterCheck),
        (status = 404, description = "Resource not found"),
        (status = 503, description = "Service not configured"),
    ),
    tag = "chapter_check",
    security(("bearer_auth" = []))
)]
pub async fn trigger_chapter_check(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ChapterCheck>, ApiError> {
    let svc = state
        .chapter_check_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("chapter check service not configured"))?;

    let result = svc.check_resource(id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/web-readers/{id}/checks",
    params(("id" = Uuid, Path, description = "Web reader resource ID")),
    responses(
        (status = 200, description = "Check history", body = Vec<ChapterCheck>),
        (status = 404, description = "Resource not found"),
    ),
    tag = "chapter_check",
    security(("bearer_auth" = []))
)]
pub async fn list_chapter_checks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ChapterCheck>>, ApiError> {
    let svc = state
        .chapter_check_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("chapter check service not configured"))?;

    let checks = svc.list_check_history(id).await?;
    Ok(Json(checks))
}
