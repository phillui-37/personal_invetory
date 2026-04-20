use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::{DomainError, Resource, ResourceLocation, WebReaderMeta};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use services::{NewLocationInput, NewWebReaderInput, UpdateWebReaderInput, WebReaderDetail};
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
pub struct WebReaderDetailResponse {
    pub resource: Resource,
    pub meta: WebReaderMeta,
    pub locations: Vec<ResourceLocation>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertWebReaderRequest {
    pub title: String,
    pub notes: Option<String>,
    pub url: String,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
    pub check_interval_secs: Option<u64>,
    pub progress_css_selector: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWebReaderRequest {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
    pub check_interval_secs: Option<u64>,
    pub progress_css_selector: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddLocationRequest {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/web-readers/list",
    params(
        ("tag" = Option<String>, Query, description = "Optional single tag name filter (deprecated, use tags)"),
        ("tags" = Vec<String>, Query, description = "Optional multiple tag names for filtering"),
        ("logic" = Option<String>, Query, description = "Filter logic: 'and' or 'or' (default: 'and')")
    ),
    responses(
        (status = 200, description = "List of web readers", body = Value),
    ),
    tag = "web_readers",
    security(("bearer_auth" = []))
)]
pub async fn list_web_readers(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let allowed_ids = resolve_list_query_filters(&state, &query).await?;
    let mut result = state
        .web_reader_service
        .list_web_readers(allowed_ids.as_ref())
        .await?;

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
    path = "/api/v1/inventory/web-readers/search",
    params(("q" = String, Query, description = "Search query")),
    responses(
        (status = 200, description = "Search results", body = Vec<Resource>),
        (status = 400, description = "Empty query"),
    ),
    tag = "web_readers",
    security(("bearer_auth" = []))
)]
pub async fn search_web_readers(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let q = query.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Err(ApiError::from(DomainError::ValidationError(
            "q must not be empty".to_string(),
        )));
    }

    let result = state.web_reader_service.search_web_readers(&q).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/web-readers/add",
    request_body = UpsertWebReaderRequest,
    responses(
        (status = 200, description = "Created web reader", body = WebReaderDetailResponse),
        (status = 409, description = "Title conflict"),
    ),
    tag = "web_readers",
    security(("bearer_auth" = []))
)]
pub async fn add_web_reader(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpsertWebReaderRequest>,
) -> Result<Json<WebReaderDetailResponse>, ApiError> {
    let detail = state
        .web_reader_service
        .add_web_reader(NewWebReaderInput {
            title: request.title,
            notes: request.notes,
            url: request.url,
            site_name: request.site_name,
            last_checked_chapter: request.last_checked_chapter,
            check_interval_secs: request.check_interval_secs,
            progress_css_selector: request.progress_css_selector,
        })
        .await?;

    Ok(Json(map_web_reader_detail(detail)))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/web-readers/{id}/detail",
    params(("id" = Uuid, Path, description = "Web reader resource ID")),
    responses(
        (status = 200, description = "Web reader detail", body = WebReaderDetailResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "web_readers",
    security(("bearer_auth" = []))
)]
pub async fn web_reader_detail(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<Json<WebReaderDetailResponse>, ApiError> {
    let detail = state
        .web_reader_service
        .web_reader_detail(resource_id)
        .await?;
    Ok(Json(map_web_reader_detail(detail)))
}

#[utoipa::path(
    put,
    path = "/api/v1/inventory/web-readers/{id}/update",
    params(("id" = Uuid, Path, description = "Web reader resource ID")),
    request_body = UpdateWebReaderRequest,
    responses(
        (status = 200, description = "Updated web reader", body = WebReaderDetailResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "web_readers",
    security(("bearer_auth" = []))
)]
pub async fn update_web_reader(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<UpdateWebReaderRequest>,
) -> Result<Json<WebReaderDetailResponse>, ApiError> {
    let detail = state
        .web_reader_service
        .update_web_reader(
            resource_id,
            UpdateWebReaderInput {
                title: request.title,
                notes: request.notes,
                url: request.url,
                site_name: request.site_name,
                last_checked_chapter: request.last_checked_chapter,
                check_interval_secs: request.check_interval_secs,
                progress_css_selector: request.progress_css_selector,
            },
        )
        .await?;

    Ok(Json(map_web_reader_detail(detail)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/inventory/web-readers/{id}/delete",
    params(("id" = Uuid, Path, description = "Web reader resource ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 404, description = "Not found"),
    ),
    tag = "web_readers",
    security(("bearer_auth" = []))
)]
pub async fn delete_web_reader(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .web_reader_service
        .delete_web_reader(resource_id)
        .await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/web-readers/{id}/locations/add",
    params(("id" = Uuid, Path, description = "Web reader resource ID")),
    request_body = AddLocationRequest,
    responses(
        (status = 200, description = "Added location", body = ResourceLocation),
        (status = 404, description = "Not found"),
    ),
    tag = "web_readers",
    security(("bearer_auth" = []))
)]
pub async fn add_web_reader_location(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<AddLocationRequest>,
) -> Result<Json<ResourceLocation>, ApiError> {
    let location = state
        .web_reader_service
        .add_web_reader_location(
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
    path = "/api/v1/inventory/web-readers/{id}/locations/{loc_id}/remove",
    params(
        ("id" = Uuid, Path, description = "Web reader resource ID"),
        ("loc_id" = Uuid, Path, description = "Location ID"),
    ),
    responses(
        (status = 200, description = "Removed"),
        (status = 404, description = "Not found"),
    ),
    tag = "web_readers",
    security(("bearer_auth" = []))
)]
pub async fn remove_web_reader_location(
    State(state): State<Arc<AppState>>,
    Path((resource_id, location_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .web_reader_service
        .remove_web_reader_location(resource_id, location_id)
        .await?;
    Ok(StatusCode::OK)
}

fn map_web_reader_detail(detail: WebReaderDetail) -> WebReaderDetailResponse {
    WebReaderDetailResponse {
        resource: detail.resource,
        meta: detail.meta,
        locations: detail.locations,
    }
}
