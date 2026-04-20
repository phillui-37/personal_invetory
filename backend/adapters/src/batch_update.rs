use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use services::{
    UpdateEbookInput, UpdateGameInput, UpdateImageInput, UpdateVideoInput, UpdateWebReaderInput,
};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{ApiError, AppState};
use super::ebook::UpdateEbookRequest;
use super::image::UpdateImageRequest;
use super::video::UpdateVideoRequest;
use super::game::UpdateGameRequest;
use super::web_reader::UpdateWebReaderRequest;

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchUpdateRequest {
    /// UUIDs of resources to update.
    pub ids: Vec<String>,
    /// Sparse field map — same shape as the single-resource update request for each type.
    pub fields: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchUpdateResponse {
    pub updated: usize,
    pub failed: Vec<BatchOpFailure>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchOpFailure {
    pub id: String,
    pub reason: String,
}

fn parse_ids(raw: &[String]) -> (Vec<Uuid>, Vec<BatchOpFailure>) {
    let mut ids = Vec::new();
    let mut failed = Vec::new();
    for s in raw {
        match Uuid::parse_str(s) {
            Ok(id) => ids.push(id),
            Err(_) => failed.push(BatchOpFailure {
                id: s.clone(),
                reason: "invalid UUID".to_string(),
            }),
        }
    }
    (ids, failed)
}

fn service_failures(raw: Vec<(Uuid, String)>) -> Vec<BatchOpFailure> {
    raw.into_iter()
        .map(|(id, reason)| BatchOpFailure { id: id.to_string(), reason })
        .collect()
}

pub async fn batch_update_ebooks(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchUpdateRequest>,
) -> Result<Json<BatchUpdateResponse>, ApiError> {
    let fields: UpdateEbookRequest = serde_json::from_value(req.fields)
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let input = UpdateEbookInput {
        title: fields.title,
        notes: fields.notes,
        author: fields.author,
        isbn: fields.isbn,
        publisher: fields.publisher,
        language: fields.language,
        file_format: fields.file_format,
    };
    let (ids, mut failed) = parse_ids(&req.ids);
    let (updated, svc_failed) = state.ebook_service.batch_update_ebooks(ids, input).await;
    failed.extend(service_failures(svc_failed));
    Ok(Json(BatchUpdateResponse { updated, failed }))
}

pub async fn batch_update_images(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchUpdateRequest>,
) -> Result<Json<BatchUpdateResponse>, ApiError> {
    let fields: UpdateImageRequest = serde_json::from_value(req.fields)
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let input = UpdateImageInput {
        title: fields.title,
        notes: fields.notes,
        width: fields.width,
        height: fields.height,
        file_format: fields.file_format,
        file_size_bytes: fields.file_size_bytes,
    };
    let (ids, mut failed) = parse_ids(&req.ids);
    let (updated, svc_failed) = state.image_service.batch_update_images(ids, input).await;
    failed.extend(service_failures(svc_failed));
    Ok(Json(BatchUpdateResponse { updated, failed }))
}

pub async fn batch_update_videos(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchUpdateRequest>,
) -> Result<Json<BatchUpdateResponse>, ApiError> {
    let fields: UpdateVideoRequest = serde_json::from_value(req.fields)
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let input = UpdateVideoInput {
        title: fields.title,
        notes: fields.notes,
        duration_secs: fields.duration_secs,
        file_format: fields.file_format,
        resolution: fields.resolution,
        file_size_bytes: fields.file_size_bytes,
    };
    let (ids, mut failed) = parse_ids(&req.ids);
    let (updated, svc_failed) = state.video_service.batch_update_videos(ids, input).await;
    failed.extend(service_failures(svc_failed));
    Ok(Json(BatchUpdateResponse { updated, failed }))
}

pub async fn batch_update_games(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchUpdateRequest>,
) -> Result<Json<BatchUpdateResponse>, ApiError> {
    let fields: UpdateGameRequest = serde_json::from_value(req.fields)
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let input = UpdateGameInput {
        title: fields.title,
        notes: fields.notes,
        platform: fields.platform,
        store: fields.store,
        developer: fields.developer,
        publisher: fields.publisher,
        manual_notes: fields.manual_notes,
    };
    let (ids, mut failed) = parse_ids(&req.ids);
    let (updated, svc_failed) = state.game_service.batch_update_games(ids, input).await;
    failed.extend(service_failures(svc_failed));
    Ok(Json(BatchUpdateResponse { updated, failed }))
}

pub async fn batch_update_web_readers(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchUpdateRequest>,
) -> Result<Json<BatchUpdateResponse>, ApiError> {
    let fields: UpdateWebReaderRequest = serde_json::from_value(req.fields)
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let input = UpdateWebReaderInput {
        title: fields.title,
        notes: fields.notes,
        url: fields.url,
        site_name: fields.site_name,
        last_checked_chapter: fields.last_checked_chapter,
        check_interval_secs: fields.check_interval_secs,
        progress_css_selector: fields.progress_css_selector,
    };
    let (ids, mut failed) = parse_ids(&req.ids);
    let (updated, svc_failed) = state.web_reader_service.batch_update_web_readers(ids, input).await;
    failed.extend(service_failures(svc_failed));
    Ok(Json(BatchUpdateResponse { updated, failed }))
}
