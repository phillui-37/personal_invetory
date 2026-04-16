use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::{DomainError, Resource, ResourceLocation};
use serde::{Deserialize, Serialize};
use services::{EbookDetail, NewEbookInput, NewLocationInput, UpdateEbookInput};
use uuid::Uuid;

use crate::{ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EbookDetailResponse {
    pub resource: Resource,
    pub meta: domain::EbookMeta,
    pub locations: Vec<ResourceLocation>,
}

#[derive(Debug, Deserialize)]
pub struct AddEbookRequest {
    pub title: String,
    pub notes: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEbookRequest {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddLocationRequest {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

pub async fn list_ebooks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let result = state.ebook_service.list_ebooks().await?;
    Ok(Json(result))
}

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

pub async fn ebook_detail(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<Json<EbookDetailResponse>, ApiError> {
    let detail = state.ebook_service.ebook_detail(resource_id).await?;
    Ok(Json(map_ebook_detail(detail)))
}

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

pub async fn delete_ebook(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.ebook_service.delete_ebook(resource_id).await?;
    Ok(StatusCode::OK)
}

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
