use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct DedupWarningResponse {
    pub id: Uuid,
    pub resource_id_a: Uuid,
    pub resource_id_b: Uuid,
    pub similarity_score: f64,
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MergeResponse {
    pub kept_resource_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MergeRequest {
    pub keep_id: Uuid,
    pub discard_id: Uuid,
}

impl From<domain::dedup::DedupWarning> for DedupWarningResponse {
    fn from(w: domain::dedup::DedupWarning) -> Self {
        let status = match w.status {
            domain::dedup::DedupWarningStatus::Pending => "pending",
            domain::dedup::DedupWarningStatus::Dismissed => "dismissed",
            domain::dedup::DedupWarningStatus::Merged => "merged",
        };
        Self {
            id: w.id,
            resource_id_a: w.resource_id_a,
            resource_id_b: w.resource_id_b,
            similarity_score: w.similarity_score,
            status: status.to_string(),
        }
    }
}

pub async fn scan_duplicates(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.dedup_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("dedup not configured".into())))?;
    let warnings: Vec<DedupWarningResponse> = svc.scan_for_duplicates()
        .await.map_err(ApiError::from)?
        .into_iter().map(Into::into).collect();
    Ok(Json(warnings))
}

pub async fn list_pending_warnings(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.dedup_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("dedup not configured".into())))?;
    let warnings: Vec<DedupWarningResponse> = svc.list_pending()
        .await.map_err(ApiError::from)?
        .into_iter().map(Into::into).collect();
    Ok(Json(warnings))
}

pub async fn dismiss_warning(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.dedup_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("dedup not configured".into())))?;
    svc.dismiss(id).await.map_err(ApiError::from)?;
    Ok(StatusCode::OK)
}

pub async fn merge_resources(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<MergeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.dedup_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("dedup not configured".into())))?;
    let merged = svc.merge(id, body.keep_id, body.discard_id)
        .await.map_err(ApiError::from)?;
    Ok((StatusCode::OK, Json(MergeResponse { kept_resource_id: merged.id })))
}
