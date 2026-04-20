use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::{DomainError, EbookMeta, Resource, ResourceLocation};
use serde::{Deserialize, Serialize};
use services::{EbookDetail, NewEbookInput, NewLocationInput, UpdateEbookInput};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    tag_filter::{resolve_list_query_filters, ListQuery},
    ApiError, AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EbookDetailResponse {
    pub resource: Resource,
    pub meta: EbookMeta,
    pub locations: Vec<ResourceLocation>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddEbookRequest {
    pub title: String,
    pub notes: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEbookRequest {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddLocationRequest {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/ebooks/list",
    params(
        ("tag" = Option<String>, Query, description = "Optional single tag name filter (deprecated, use tags)"),
        ("tags" = Vec<String>, Query, description = "Optional multiple tag names for filtering"),
        ("logic" = Option<String>, Query, description = "Filter logic: 'and' or 'or' (default: 'and')")
    ),
    responses(
        (status = 200, description = "List of ebooks", body = Vec<Resource>),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn list_ebooks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let allowed_ids = resolve_list_query_filters(&state, &query).await?;
    let result = state.ebook_service.list_ebooks(allowed_ids.as_ref()).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/ebooks/search",
    params(("q" = String, Query, description = "Search query")),
    responses(
        (status = 200, description = "Search results", body = Vec<Resource>),
        (status = 400, description = "Empty query"),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn search_ebooks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let q = query.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Err(ApiError::from(DomainError::ValidationError(
            "q must not be empty".to_string(),
        )));
    }

    let result = state.ebook_service.search_ebooks(&q).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/ebooks/add",
    request_body = AddEbookRequest,
    responses(
        (status = 200, description = "Created ebook detail", body = EbookDetailResponse),
        (status = 409, description = "Title conflict"),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn add_ebook(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AddEbookRequest>,
) -> Result<Json<EbookDetailResponse>, ApiError> {
    let detail = state
        .ebook_service
        .add_ebook(NewEbookInput {
            title: request.title,
            notes: request.notes,
            author: request.author,
            isbn: request.isbn,
            publisher: request.publisher,
            language: request.language,
            file_format: request.file_format,
        })
        .await?;

    Ok(Json(map_ebook_detail(detail)))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/ebooks/{id}/detail",
    params(("id" = Uuid, Path, description = "Ebook resource ID")),
    responses(
        (status = 200, description = "Ebook detail", body = EbookDetailResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn ebook_detail(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<Json<EbookDetailResponse>, ApiError> {
    let detail = state.ebook_service.ebook_detail(resource_id).await?;
    Ok(Json(map_ebook_detail(detail)))
}

#[utoipa::path(
    put,
    path = "/api/v1/inventory/ebooks/{id}/update",
    params(("id" = Uuid, Path, description = "Ebook resource ID")),
    request_body = UpdateEbookRequest,
    responses(
        (status = 200, description = "Updated ebook detail", body = EbookDetailResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn update_ebook(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<UpdateEbookRequest>,
) -> Result<Json<EbookDetailResponse>, ApiError> {
    let detail = state
        .ebook_service
        .update_ebook(
            resource_id,
            UpdateEbookInput {
                title: request.title,
                notes: request.notes,
                author: request.author,
                isbn: request.isbn,
                publisher: request.publisher,
                language: request.language,
                file_format: request.file_format,
            },
        )
        .await?;

    Ok(Json(map_ebook_detail(detail)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/inventory/ebooks/{id}/delete",
    params(("id" = Uuid, Path, description = "Ebook resource ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 404, description = "Not found"),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn delete_ebook(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.ebook_service.delete_ebook(resource_id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/ebooks/{id}/locations/add",
    params(("id" = Uuid, Path, description = "Ebook resource ID")),
    request_body = AddLocationRequest,
    responses(
        (status = 200, description = "Added location", body = ResourceLocation),
        (status = 404, description = "Not found"),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn add_ebook_location(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<AddLocationRequest>,
) -> Result<Json<ResourceLocation>, ApiError> {
    let location = state
        .ebook_service
        .add_ebook_location(
            resource_id,
            NewLocationInput {
                device_id: request.device_id,
                path_or_url: request.path_or_url,
                storage_type: request.storage_type,
            },
        )
        .await?;

    Ok(Json(location))
}

#[utoipa::path(
    delete,
    path = "/api/v1/inventory/ebooks/{id}/locations/{loc_id}/remove",
    params(
        ("id" = Uuid, Path, description = "Ebook resource ID"),
        ("loc_id" = Uuid, Path, description = "Location ID"),
    ),
    responses(
        (status = 200, description = "Removed"),
        (status = 404, description = "Not found"),
    ),
    tag = "ebooks",
    security(("bearer_auth" = []))
)]
pub async fn remove_ebook_location(
    State(state): State<Arc<AppState>>,
    Path((resource_id, location_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .ebook_service
        .remove_ebook_location(resource_id, location_id)
        .await?;
    Ok(StatusCode::OK)
}

fn map_ebook_detail(detail: EbookDetail) -> EbookDetailResponse {
    EbookDetailResponse {
        resource: detail.resource,
        meta: detail.meta,
        locations: detail.locations,
    }
}
