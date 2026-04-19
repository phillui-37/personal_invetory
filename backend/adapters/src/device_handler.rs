use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceResponse {
    pub id: String,
    pub device_id: String,
    pub device_name: String,
    pub linked_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delinked_at: Option<String>,
    pub location_count: u64,
    pub is_current: bool,
}

impl From<services::DeviceInfo> for DeviceResponse {
    fn from(d: services::DeviceInfo) -> Self {
        Self {
            id: d.id,
            device_id: d.device_id,
            device_name: d.device_name,
            linked_at: d.linked_at.to_rfc3339(),
            delinked_at: d.delinked_at.map(|t| t.to_rfc3339()),
            location_count: d.location_count,
            is_current: d.is_current,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterDeviceRequest {
    pub device_id: String,
    pub device_name: Option<String>,
}

fn device_svc(state: &AppState) -> Result<&Arc<services::DeviceService>, ApiError> {
    state
        .device_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("device service not configured"))
}

/// List all registered devices (active and delinked) with location counts.
#[utoipa::path(
    get,
    path = "/api/v1/devices",
    responses(
        (status = 200, description = "Device list", body = Vec<DeviceResponse>),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn list_devices(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = device_svc(&state)?;
    let devices = svc.list().await.map_err(ApiError::from)?;
    Ok(Json(
        devices
            .into_iter()
            .map(DeviceResponse::from)
            .collect::<Vec<_>>(),
    ))
}

/// Return the current backend device (identified by DEVICE_ID env var).
#[utoipa::path(
    get,
    path = "/api/v1/devices/current",
    responses(
        (status = 200, description = "Current device", body = DeviceResponse),
        (status = 404, description = "Current device not registered"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn current_device(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = device_svc(&state)?;
    let device = svc.current().await.map_err(ApiError::from)?;
    Ok(Json(DeviceResponse::from(device)))
}

/// Register a device. Re-registering delinks the old binding and creates a new one.
#[utoipa::path(
    post,
    path = "/api/v1/devices/register",
    request_body = RegisterDeviceRequest,
    responses(
        (status = 201, description = "Device registered", body = DeviceResponse),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn register_device(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterDeviceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = device_svc(&state)?;
    let info = svc
        .register(&body.device_id, body.device_name.as_deref())
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(DeviceResponse::from(info))))
}

/// Delink a device by its device_id. Cannot delink the current device.
#[utoipa::path(
    post,
    path = "/api/v1/devices/{device_id}/delink",
    params(("device_id" = String, Path, description = "Device ID to delink")),
    responses(
        (status = 200, description = "Device delinked"),
        (status = 404, description = "Device not found"),
        (status = 409, description = "Already delinked"),
        (status = 422, description = "Cannot delink current device"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn delink_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = device_svc(&state)?;
    svc.delink(&device_id).await.map_err(ApiError::from)?;
    Ok(StatusCode::OK)
}
