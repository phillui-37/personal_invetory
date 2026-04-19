use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{ApiError, AppState};

#[derive(Debug, Deserialize, ToSchema)]
pub struct InitializeVaultRequest {
    pub master_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UnlockVaultRequest {
    pub master_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StoreCredentialRequest {
    pub platform: String,
    pub credential_type: String,
    pub plaintext: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RetrieveCredentialRequest {
    pub platform: String,
    pub credential_type: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteCredentialRequest {
    pub platform: String,
    pub credential_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VaultStatusResponse {
    pub initialized: bool,
    pub unlocked: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CredentialResponse {
    pub plaintext: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlatformListResponse {
    pub platforms: Vec<String>,
}

pub async fn vault_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.vault_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("vault not configured".into())))?;
    let initialized = svc.is_initialized().await.map_err(ApiError::from)?;
    let unlocked = svc.is_unlocked();
    Ok(Json(VaultStatusResponse { initialized, unlocked }))
}

pub async fn initialize_vault(
    State(state): State<Arc<AppState>>,
    Json(body): Json<InitializeVaultRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.vault_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("vault not configured".into())))?;
    svc.initialize(&body.master_password).await.map_err(ApiError::from)?;
    Ok(StatusCode::CREATED)
}

pub async fn unlock_vault(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UnlockVaultRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.vault_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("vault not configured".into())))?;
    svc.unlock(&body.master_password).await.map_err(ApiError::from)?;
    Ok(StatusCode::OK)
}

pub async fn lock_vault(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.vault_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("vault not configured".into())))?;
    svc.lock();
    Ok(StatusCode::OK)
}

pub async fn store_credential(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StoreCredentialRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.vault_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("vault not configured".into())))?;
    svc.store(&body.platform, &body.credential_type, body.plaintext.as_bytes())
        .await.map_err(ApiError::from)?;
    Ok(StatusCode::CREATED)
}

pub async fn retrieve_credential(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RetrieveCredentialRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.vault_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("vault not configured".into())))?;
    let plaintext = svc.retrieve(&body.platform, &body.credential_type)
        .await.map_err(ApiError::from)?;
    let text = String::from_utf8(plaintext)
        .map_err(|e| ApiError::from(domain::DomainError::InternalError(format!("invalid utf8: {e}"))))?;
    Ok(Json(CredentialResponse { plaintext: text }))
}

pub async fn delete_credential(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DeleteCredentialRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.vault_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("vault not configured".into())))?;
    svc.delete(&body.platform, &body.credential_type)
        .await.map_err(ApiError::from)?;
    Ok(StatusCode::OK)
}

pub async fn list_vault_platforms(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = state.vault_service.as_ref()
        .ok_or_else(|| ApiError::from(domain::DomainError::InternalError("vault not configured".into())))?;
    let platforms = svc.list_platforms().await.map_err(ApiError::from)?;
    Ok(Json(PlatformListResponse { platforms }))
}
