use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::DomainError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTagRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AttachTagRequest {
    pub tag_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TagResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

impl From<domain::tag::Tag> for TagResponse {
    fn from(t: domain::tag::Tag) -> Self {
        Self {
            id: t.id,
            name: t.name,
            created_at: t.created_at.to_rfc3339(),
        }
    }
}

fn tag_svc(state: &AppState) -> Result<&Arc<services::TagService>, ApiError> {
    state
        .tag_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("tag service not configured"))
}

async fn ensure_resource_type(
    state: &AppState,
    resource_type: &str,
    id: Uuid,
) -> Result<(), ApiError> {
    match resource_type {
        "ebooks" => state
            .ebook_service
            .ebook_detail(id)
            .await
            .map(|_| ())
            .map_err(ApiError::from),
        "web-readers" => state
            .web_reader_service
            .web_reader_detail(id)
            .await
            .map(|_| ())
            .map_err(ApiError::from),
        "images" => state
            .image_service
            .image_detail(id)
            .await
            .map(|_| ())
            .map_err(ApiError::from),
        "videos" => state
            .video_service
            .video_detail(id)
            .await
            .map(|_| ())
            .map_err(ApiError::from),
        "games" => state
            .game_service
            .game_detail(id)
            .await
            .map(|_| ())
            .map_err(ApiError::from),
        other => Err(ApiError::from(DomainError::ValidationError(format!(
            "unsupported resource type '{other}'"
        )))),
    }
}

/// List all tags.
#[utoipa::path(
    get,
    path = "/api/v1/tags",
    responses(
        (status = 200, description = "List of tags", body = Vec<TagResponse>),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = tag_svc(&state)?;
    let tags = svc.list().await.map_err(ApiError::from)?;
    Ok(Json(
        tags.into_iter().map(TagResponse::from).collect::<Vec<_>>(),
    ))
}

/// Create a new tag. Name is trimmed and lowercased.
#[utoipa::path(
    post,
    path = "/api/v1/tags",
    request_body = CreateTagRequest,
    responses(
        (status = 201, description = "Tag created", body = TagResponse),
        (status = 409, description = "Tag name already exists"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTagRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = tag_svc(&state)?;
    let tag = svc.create(&body.name).await.map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(TagResponse::from(tag))))
}

/// Delete a tag by ID.
#[utoipa::path(
    delete,
    path = "/api/v1/tags/{id}",
    params(("id" = String, Path, description = "Tag ID")),
    responses(
        (status = 204, description = "Tag deleted"),
        (status = 404, description = "Tag not found"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn delete_tag(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = tag_svc(&state)?;
    svc.delete(&id).await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// List tags attached to a resource.
#[utoipa::path(
    get,
    path = "/api/v1/inventory/{resource_type}/{id}/tags",
    params(
        ("resource_type" = String, Path, description = "Resource type (ebooks, web-readers, images, videos, games)"),
        ("id" = Uuid, Path, description = "Resource UUID"),
    ),
    responses(
        (status = 200, description = "Tags for resource", body = Vec<TagResponse>),
        (status = 404, description = "Resource not found"),
        (status = 422, description = "Unsupported resource type"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn list_resource_tags(
    Path((resource_type, id)): Path<(String, Uuid)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_resource_type(&state, &resource_type, id).await?;
    let svc = tag_svc(&state)?;
    let tags = svc
        .tags_for_resource(&id.to_string())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        tags.into_iter().map(TagResponse::from).collect::<Vec<_>>(),
    ))
}

/// Attach a tag to a resource.
#[utoipa::path(
    post,
    path = "/api/v1/inventory/{resource_type}/{id}/tags",
    params(
        ("resource_type" = String, Path, description = "Resource type"),
        ("id" = Uuid, Path, description = "Resource UUID"),
    ),
    request_body = AttachTagRequest,
    responses(
        (status = 200, description = "Tag attached"),
        (status = 404, description = "Resource or tag not found"),
        (status = 422, description = "Unsupported resource type"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn attach_tag(
    Path((resource_type, id)): Path<(String, Uuid)>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AttachTagRequest>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_resource_type(&state, &resource_type, id).await?;
    let svc = tag_svc(&state)?;
    svc.attach_tag(&id.to_string(), &body.tag_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::OK)
}

/// Detach a tag from a resource.
#[utoipa::path(
    delete,
    path = "/api/v1/inventory/{resource_type}/{id}/tags/{tag_id}",
    params(
        ("resource_type" = String, Path, description = "Resource type"),
        ("id" = Uuid, Path, description = "Resource UUID"),
        ("tag_id" = String, Path, description = "Tag ID"),
    ),
    responses(
        (status = 204, description = "Tag detached"),
        (status = 404, description = "Resource or association not found"),
        (status = 422, description = "Unsupported resource type"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn detach_tag(
    Path((resource_type, id, tag_id)): Path<(String, Uuid, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_resource_type(&state, &resource_type, id).await?;
    let svc = tag_svc(&state)?;
    svc.detach_tag(&id.to_string(), &tag_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
