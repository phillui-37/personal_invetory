use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use domain::ecosystem::DiscoveredItem;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct SyncJobResponse {
    pub id: Uuid,
    pub platform: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub items_found: u32,
    pub items_created: u32,
    pub items_skipped: u32,
    pub items_failed: u32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<domain::sync::SyncJob> for SyncJobResponse {
    fn from(job: domain::sync::SyncJob) -> Self {
        Self {
            id: job.id,
            platform: job.platform,
            status: format!("{:?}", job.status),
            started_at: job.started_at,
            completed_at: job.completed_at,
            items_found: job.items_found,
            items_created: job.items_created,
            items_skipped: job.items_skipped,
            items_failed: job.items_failed,
            error_message: job.error_message,
            created_at: job.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TriggerSyncResponse {
    pub job_id: Uuid,
    pub platform: String,
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SyncJobListResponse {
    pub jobs: Vec<SyncJobResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EcosystemStatusResponse {
    pub platforms: Vec<PlatformStatus>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlatformStatus {
    pub platform: String,
    pub last_sync: Option<SyncJobResponse>,
}

/// A single discovered item to import during a sync.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DiscoveredItemInput {
    pub external_id: String,
    pub title: String,
    /// Arbitrary key-value metadata (resource_type, maker, author, …).
    pub metadata: std::collections::HashMap<String, String>,
}

impl DiscoveredItemInput {
    fn into_domain(self, platform: &str) -> DiscoveredItem {
        DiscoveredItem {
            external_id: self.external_id,
            title: self.title,
            platform: platform.to_string(),
            metadata: self.metadata,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TriggerSyncRequest {
    /// Optional credentials bytes (JSON) to pass to the connector.
    pub credentials: Option<Vec<u8>>,
    /// Items discovered by the client-side connector to import.
    pub discovered_items: Option<Vec<DiscoveredItemInput>>,
}

fn sync_svc(
    state: &AppState,
) -> Result<&Arc<services::SyncService>, ApiError> {
    state
        .sync_service
        .as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("sync not configured".into())))
}

/// POST /api/v1/ecosystem/:platform/sync — triggers a sync job for the given platform.
/// Callers supply discovered items in the request body; an empty list creates a no-op job.
#[utoipa::path(
    post,
    path = "/api/v1/ecosystem/{platform}/sync",
    params(("platform" = String, Path, description = "Platform name (steam/dlsite/fanza/bookwalker/kindle)")),
    request_body = TriggerSyncRequest,
    responses(
        (status = 202, description = "Sync triggered", body = TriggerSyncResponse),
        (status = 503, description = "Sync service not configured"),
    ),
    tag = "ecosystem"
)]
pub async fn trigger_sync(
    State(state): State<Arc<AppState>>,
    Path(platform): Path<String>,
    Json(body): Json<TriggerSyncRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = sync_svc(&state)?;
    let items = body
        .discovered_items
        .unwrap_or_default()
        .into_iter()
        .map(|i| i.into_domain(&platform))
        .collect();
    let job = svc
        .sync_discovered_items(&platform, items)
        .await
        .map_err(ApiError::from)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(TriggerSyncResponse {
            job_id: job.id,
            platform: job.platform,
            status: format!("{:?}", job.status),
        }),
    ))
}

/// GET /api/v1/ecosystem/:platform/syncs — list all sync jobs for a platform.
#[utoipa::path(
    get,
    path = "/api/v1/ecosystem/{platform}/syncs",
    params(("platform" = String, Path, description = "Platform name")),
    responses(
        (status = 200, description = "List of sync jobs", body = SyncJobListResponse),
        (status = 503, description = "Sync service not configured"),
    ),
    tag = "ecosystem"
)]
pub async fn list_platform_syncs(
    State(state): State<Arc<AppState>>,
    Path(platform): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = sync_svc(&state)?;
    let jobs = svc
        .list_jobs(&platform)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(SyncJobResponse::from)
        .collect();
    Ok(Json(SyncJobListResponse { jobs }))
}

/// GET /api/v1/ecosystem/:platform/syncs/:id — get a specific sync job.
#[utoipa::path(
    get,
    path = "/api/v1/ecosystem/{platform}/syncs/{id}",
    params(
        ("platform" = String, Path, description = "Platform name"),
        ("id" = Uuid, Path, description = "Sync job UUID"),
    ),
    responses(
        (status = 200, description = "Sync job detail", body = SyncJobResponse),
        (status = 404, description = "Job not found or platform mismatch"),
        (status = 503, description = "Sync service not configured"),
    ),
    tag = "ecosystem"
)]
pub async fn get_sync_job(
    State(state): State<Arc<AppState>>,
    Path((platform, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = sync_svc(&state)?;
    let job = svc.get_job(id).await.map_err(ApiError::from)?;
    if job.platform != platform {
        return Err(ApiError::from(domain::DomainError::NotFound(
            format!("sync job {id} not found for platform {platform}"),
        )));
    }
    Ok(Json(SyncJobResponse::from(job)))
}

/// GET /api/v1/ecosystem/status — returns all known platforms with their last sync job.
#[utoipa::path(
    get,
    path = "/api/v1/ecosystem/status",
    responses(
        (status = 200, description = "Platform status overview", body = EcosystemStatusResponse),
        (status = 503, description = "Sync service not configured"),
    ),
    tag = "ecosystem"
)]
pub async fn ecosystem_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = sync_svc(&state)?;
    let known_platforms = ["steam", "dlsite", "fanza", "bookwalker", "kindle"];
    let mut platform_statuses = Vec::new();
    for platform in known_platforms {
        let jobs = svc.list_jobs(platform).await.map_err(ApiError::from)?;
        // list_jobs returns jobs in descending order; first element is the most recent.
        let last_sync = jobs.into_iter().next().map(SyncJobResponse::from);
        platform_statuses.push(PlatformStatus {
            platform: platform.to_string(),
            last_sync,
        });
    }
    Ok(Json(EcosystemStatusResponse {
        platforms: platform_statuses,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OtpSubmitRequest {
    pub platform: String,
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OtpSubmitResponse {
    pub accepted: bool,
}

/// POST /api/v1/sync/otp — submit an OTP code during a browser-based sync.
#[utoipa::path(
    post,
    path = "/api/v1/sync/otp",
    request_body = OtpSubmitRequest,
    responses(
        (status = 200, description = "OTP submitted", body = OtpSubmitResponse),
        (status = 503, description = "OTP service not configured"),
    ),
    tag = "ecosystem"
)]
pub async fn submit_otp(
    State(state): State<Arc<AppState>>,
    Json(body): Json<OtpSubmitRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let otp_svc = state
        .otp_service
        .as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("OTP service not configured".into())))?;
    let accepted = otp_svc.submit_otp(&body.platform, &body.code).await.is_ok();
    Ok(Json(OtpSubmitResponse { accepted }))
}

#[cfg(test)]
mod otp_tests {
    use super::*;

    #[test]
    fn otp_submit_request_deserializes() {
        let json = r#"{"platform": "kindle", "code": "123456"}"#;
        let req: OtpSubmitRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.platform, "kindle");
        assert_eq!(req.code, "123456");
    }

    #[test]
    fn otp_submit_response_serializes() {
        let resp = OtpSubmitResponse { accepted: true };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("true"));
    }
}
