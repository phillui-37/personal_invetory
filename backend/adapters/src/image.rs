use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::{DomainError, ImageMeta, Resource, ResourceLocation};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use services::{ImageDetail, NewImageInput, NewLocationInput, UpdateImageInput};
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
pub struct ImageDetailResponse {
    pub resource: Resource,
    pub meta: ImageMeta,
    pub locations: Vec<ResourceLocation>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddImageRequest {
    pub title: String,
    pub notes: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_format: Option<String>,
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateImageRequest {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_format: Option<String>,
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddLocationRequest {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/images/list",
    params(
        ("tag" = Option<String>, Query, description = "Optional single tag name filter (deprecated, use tags)"),
        ("tags" = Vec<String>, Query, description = "Optional multiple tag names for filtering"),
        ("logic" = Option<String>, Query, description = "Filter logic: 'and' or 'or' (default: 'and')")
    ),
    responses(
        (status = 200, description = "List of images", body = Value),
    ),
    tag = "images",
    security(("bearer_auth" = []))
)]
pub async fn list_images(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let allowed_ids = resolve_list_query_filters(&state, &query).await?;
    let mut result = state.image_service.list_images(allowed_ids.as_ref()).await?;

    let (sort_field, sort_order) = crate::tag_filter::resolve_sort_params(
        query.sort_by.as_deref(),
        query.sort_order.as_deref(),
    );
    result = services::sort_resources(
        result,
        match sort_field {
            crate::search_options::SortField::Title => services::SortField::Title,
            crate::search_options::SortField::DateAdded => services::SortField::DateAdded,
        },
        match sort_order {
            crate::search_options::SortOrder::Asc => services::SortOrder::Asc,
            crate::search_options::SortOrder::Desc => services::SortOrder::Desc,
        },
    );

    if query.with_facets == Some(true) {
        let facets = services::count_formats(&result);
        let facet_response: Vec<serde_json::Value> = facets
            .into_iter()
            .map(|f| serde_json::json!({"name": f.name, "count": f.count}))
            .collect();
        Ok(Json(serde_json::json!({
            "items": result,
            "facets": {"formats": facet_response}
        })))
    } else {
        Ok(Json(serde_json::to_value(&result).map_err(|e| {
            ApiError::from(domain::DomainError::InternalError(format!("JSON error: {e}")))
        })?))
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/images/search",
    params(("q" = String, Query, description = "Search query")),
    responses(
        (status = 200, description = "Search results", body = Vec<Resource>),
        (status = 400, description = "Empty query"),
    ),
    tag = "images",
    security(("bearer_auth" = []))
)]
pub async fn search_images(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let q = query.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Err(ApiError::from(DomainError::ValidationError(
            "q must not be empty".to_string(),
        )));
    }

    let result = state.image_service.search_images(&q).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/images/add",
    request_body = AddImageRequest,
    responses(
        (status = 200, description = "Created image detail", body = ImageDetailResponse),
        (status = 409, description = "Title conflict"),
    ),
    tag = "images",
    security(("bearer_auth" = []))
)]
pub async fn add_image(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AddImageRequest>,
) -> Result<Json<ImageDetailResponse>, ApiError> {
    let detail = state
        .image_service
        .add_image(NewImageInput {
            title: request.title,
            notes: request.notes,
            width: request.width,
            height: request.height,
            file_format: request.file_format,
            file_size_bytes: request.file_size_bytes,
        })
        .await?;

    Ok(Json(map_image_detail(detail)))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/images/{id}/detail",
    params(("id" = Uuid, Path, description = "Image resource ID")),
    responses(
        (status = 200, description = "Image detail", body = ImageDetailResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "images",
    security(("bearer_auth" = []))
)]
pub async fn image_detail(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<Json<ImageDetailResponse>, ApiError> {
    let detail = state.image_service.image_detail(resource_id).await?;
    Ok(Json(map_image_detail(detail)))
}

#[utoipa::path(
    put,
    path = "/api/v1/inventory/images/{id}/update",
    params(("id" = Uuid, Path, description = "Image resource ID")),
    request_body = UpdateImageRequest,
    responses(
        (status = 200, description = "Updated image detail", body = ImageDetailResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "images",
    security(("bearer_auth" = []))
)]
pub async fn update_image(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<UpdateImageRequest>,
) -> Result<Json<ImageDetailResponse>, ApiError> {
    let detail = state
        .image_service
        .update_image(
            resource_id,
            UpdateImageInput {
                title: request.title,
                notes: request.notes,
                width: request.width,
                height: request.height,
                file_format: request.file_format,
                file_size_bytes: request.file_size_bytes,
            },
        )
        .await?;

    Ok(Json(map_image_detail(detail)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/inventory/images/{id}/delete",
    params(("id" = Uuid, Path, description = "Image resource ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 404, description = "Not found"),
    ),
    tag = "images",
    security(("bearer_auth" = []))
)]
pub async fn delete_image(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.image_service.delete_image(resource_id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/images/{id}/locations/add",
    params(("id" = Uuid, Path, description = "Image resource ID")),
    request_body = AddLocationRequest,
    responses(
        (status = 200, description = "Added location", body = ResourceLocation),
        (status = 404, description = "Not found"),
    ),
    tag = "images",
    security(("bearer_auth" = []))
)]
pub async fn add_image_location(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<AddLocationRequest>,
) -> Result<Json<ResourceLocation>, ApiError> {
    let location = state
        .image_service
        .add_image_location(
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
    path = "/api/v1/inventory/images/{id}/locations/{loc_id}/remove",
    params(
        ("id" = Uuid, Path, description = "Image resource ID"),
        ("loc_id" = Uuid, Path, description = "Location ID"),
    ),
    responses(
        (status = 200, description = "Removed"),
        (status = 404, description = "Not found"),
    ),
    tag = "images",
    security(("bearer_auth" = []))
)]
pub async fn remove_image_location(
    State(state): State<Arc<AppState>>,
    Path((resource_id, location_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .image_service
        .remove_image_location(resource_id, location_id)
        .await?;
    Ok(StatusCode::OK)
}

fn map_image_detail(detail: ImageDetail) -> ImageDetailResponse {
    ImageDetailResponse {
        resource: detail.resource,
        meta: detail.meta,
        locations: detail.locations,
    }
}
