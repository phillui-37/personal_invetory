use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    http::StatusCode,
    Json,
};
use domain::Notification;
use serde::Deserialize;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use uuid::Uuid;

use crate::{error::ApiError, state::{AppState, NotificationEvent}};

#[derive(Deserialize)]
pub struct NotificationsQuery {
    #[serde(default)]
    pub unread_only: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications",
    params(("unread_only" = Option<bool>, Query, description = "Filter to unread only")),
    responses(
        (status = 200, description = "List of notifications", body = Vec<Notification>),
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn list_notifications(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NotificationsQuery>,
) -> Result<Json<Vec<Notification>>, ApiError> {
    let svc = state
        .chapter_check_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("chapter check service not configured"))?;

    let notifications = svc.list_notifications(params.unread_only).await?;
    Ok(Json(notifications))
}

#[utoipa::path(
    post,
    path = "/api/v1/notifications/{id}/read",
    params(("id" = Uuid, Path, description = "Notification ID")),
    responses(
        (status = 204, description = "Marked as read"),
        (status = 404, description = "Notification not found"),
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn mark_notification_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let svc = state
        .chapter_check_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("chapter check service not configured"))?;

    svc.mark_notification_read(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications/stream",
    responses(
        (status = 200, description = "SSE stream of notification events", content_type = "text/event-stream"),
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn notifications_stream(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let rx = state.notification_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| {
            result.ok().map(|event: NotificationEvent| {
                let data = serde_json::json!({
                    "id": event.id,
                    "resource_id": event.resource_id,
                    "message": event.message,
                });
                Ok::<Event, Infallible>(Event::default().data(data.to_string()))
            })
        });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("ping"),
    )
}
