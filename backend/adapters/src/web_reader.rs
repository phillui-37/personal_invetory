use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::{DomainError, Resource, ResourceLocation};
use serde::{Deserialize, Serialize};
use services::{NewLocationInput, NewWebReaderInput, WebReaderDetail};
use uuid::Uuid;

use crate::{ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WebReaderDetailResponse {
    pub resource: Resource,
    pub meta: domain::WebReaderMeta,
    pub locations: Vec<ResourceLocation>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertWebReaderRequest {
    pub title: String,
    pub notes: Option<String>,
    pub url: String,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddLocationRequest {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

pub async fn list_web_readers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let result = state.web_reader_service.list_web_readers().await?;
    Ok(Json(result))
}

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
        })
        .await?;

    Ok(Json(map_web_reader_detail(detail)))
}

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

pub async fn update_web_reader(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<UpsertWebReaderRequest>,
) -> Result<Json<WebReaderDetailResponse>, ApiError> {
    let detail = state
        .web_reader_service
        .update_web_reader(
            resource_id,
            NewWebReaderInput {
                title: request.title,
                notes: request.notes,
                url: request.url,
                site_name: request.site_name,
                last_checked_chapter: request.last_checked_chapter,
            },
        )
        .await?;

    Ok(Json(map_web_reader_detail(detail)))
}

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
