use std::collections::HashSet;

use domain::DomainError;
use serde::Deserialize;
use uuid::Uuid;

use crate::{ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
}

pub async fn resolve_tag_filter_ids(
    state: &AppState,
    tag_filter: Option<&str>,
) -> Result<Option<HashSet<Uuid>>, ApiError> {
    let Some(tag_name) = tag_filter.map(str::trim).filter(|tag| !tag.is_empty()) else {
        return Ok(None);
    };

    let tag_service = state
        .tag_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("tag service not configured"))?;
    let resource_ids = tag_service
        .resource_ids_for_tag_name(tag_name)
        .await
        .map_err(ApiError::from)?;

    resource_ids
        .into_iter()
        .map(|resource_id| {
            Uuid::parse_str(&resource_id).map_err(|_| {
                ApiError::from(DomainError::InternalError(format!(
                    "invalid tagged resource id '{resource_id}'"
                )))
            })
        })
        .collect::<Result<HashSet<_>, _>>()
        .map(Some)
}
