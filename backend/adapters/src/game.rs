use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::{DomainError, GameMeta, Resource, ResourceLocation};
use serde::{Deserialize, Serialize};
use services::{GameDetail, NewGameInput, NewLocationInput, UpdateGameInput};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    tag_filter::{resolve_tag_filter_ids, ListQuery},
    ApiError, AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameDetailResponse {
    pub resource: Resource,
    pub meta: GameMeta,
    pub locations: Vec<ResourceLocation>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddGameRequest {
    pub title: String,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub store: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub manual_notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateGameRequest {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub store: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub manual_notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddLocationRequest {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/games/list",
    params(("tag" = Option<String>, Query, description = "Optional tag name filter")),
    responses(
        (status = 200, description = "List of games", body = Vec<Resource>),
    ),
    tag = "games",
    security(("bearer_auth" = []))
)]
pub async fn list_games(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let allowed_ids = resolve_tag_filter_ids(&state, query.tag.as_deref()).await?;
    let result = state.game_service.list_games(allowed_ids.as_ref()).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/games/search",
    params(("q" = String, Query, description = "Search query")),
    responses(
        (status = 200, description = "Search results", body = Vec<Resource>),
        (status = 400, description = "Empty query"),
    ),
    tag = "games",
    security(("bearer_auth" = []))
)]
pub async fn search_games(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let q = query.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Err(ApiError::from(DomainError::ValidationError(
            "q must not be empty".to_string(),
        )));
    }

    let result = state.game_service.search_games(&q).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/games/add",
    request_body = AddGameRequest,
    responses(
        (status = 200, description = "Created game detail", body = GameDetailResponse),
        (status = 409, description = "Title conflict"),
    ),
    tag = "games",
    security(("bearer_auth" = []))
)]
pub async fn add_game(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AddGameRequest>,
) -> Result<Json<GameDetailResponse>, ApiError> {
    let detail = state
        .game_service
        .add_game(NewGameInput {
            title: request.title,
            notes: request.notes,
            platform: request.platform,
            store: request.store,
            developer: request.developer,
            publisher: request.publisher,
            manual_notes: request.manual_notes,
        })
        .await?;

    Ok(Json(map_game_detail(detail)))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/games/{id}/detail",
    params(("id" = Uuid, Path, description = "Game resource ID")),
    responses(
        (status = 200, description = "Game detail", body = GameDetailResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "games",
    security(("bearer_auth" = []))
)]
pub async fn game_detail(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<Json<GameDetailResponse>, ApiError> {
    let detail = state.game_service.game_detail(resource_id).await?;
    Ok(Json(map_game_detail(detail)))
}

#[utoipa::path(
    put,
    path = "/api/v1/inventory/games/{id}/update",
    params(("id" = Uuid, Path, description = "Game resource ID")),
    request_body = UpdateGameRequest,
    responses(
        (status = 200, description = "Updated game detail", body = GameDetailResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "games",
    security(("bearer_auth" = []))
)]
pub async fn update_game(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<UpdateGameRequest>,
) -> Result<Json<GameDetailResponse>, ApiError> {
    let detail = state
        .game_service
        .update_game(
            resource_id,
            UpdateGameInput {
                title: request.title,
                notes: request.notes,
                platform: request.platform,
                store: request.store,
                developer: request.developer,
                publisher: request.publisher,
                manual_notes: request.manual_notes,
            },
        )
        .await?;

    Ok(Json(map_game_detail(detail)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/inventory/games/{id}/delete",
    params(("id" = Uuid, Path, description = "Game resource ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 404, description = "Not found"),
    ),
    tag = "games",
    security(("bearer_auth" = []))
)]
pub async fn delete_game(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.game_service.delete_game(resource_id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/games/{id}/locations/add",
    params(("id" = Uuid, Path, description = "Game resource ID")),
    request_body = AddLocationRequest,
    responses(
        (status = 200, description = "Added location", body = ResourceLocation),
        (status = 404, description = "Not found"),
    ),
    tag = "games",
    security(("bearer_auth" = []))
)]
pub async fn add_game_location(
    State(state): State<Arc<AppState>>,
    Path(resource_id): Path<Uuid>,
    Json(request): Json<AddLocationRequest>,
) -> Result<Json<ResourceLocation>, ApiError> {
    let location = state
        .game_service
        .add_game_location(
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
    path = "/api/v1/inventory/games/{id}/locations/{loc_id}/remove",
    params(
        ("id" = Uuid, Path, description = "Game resource ID"),
        ("loc_id" = Uuid, Path, description = "Location ID"),
    ),
    responses(
        (status = 200, description = "Removed"),
        (status = 404, description = "Not found"),
    ),
    tag = "games",
    security(("bearer_auth" = []))
)]
pub async fn remove_game_location(
    State(state): State<Arc<AppState>>,
    Path((resource_id, location_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .game_service
        .remove_game_location(resource_id, location_id)
        .await?;
    Ok(StatusCode::OK)
}

fn map_game_detail(detail: GameDetail) -> GameDetailResponse {
    GameDetailResponse {
        resource: detail.resource,
        meta: detail.meta,
        locations: detail.locations,
    }
}
