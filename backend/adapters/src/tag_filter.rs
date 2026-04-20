use std::collections::HashSet;

use domain::DomainError;
use serde::Deserialize;
use uuid::Uuid;

use crate::{ApiError, AppState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterLogic {
    And,
    Or,
}

impl FilterLogic {
    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.trim().to_lowercase().as_str() {
            "and" => Ok(FilterLogic::And),
            "or" => Ok(FilterLogic::Or),
            _ => Err(DomainError::ValidationError(
                "filter logic must be 'and' or 'or'".into(),
            )),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub tags: Vec<String>,
    pub tag: Option<String>,
    pub logic: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub with_facets: Option<bool>,
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

pub async fn resolve_multi_tag_filter_ids(
    state: &AppState,
    tags: &[String],
    logic: FilterLogic,
) -> Result<Option<HashSet<Uuid>>, ApiError> {
    if tags.is_empty() {
        return Ok(None);
    }

    let tag_service = state
        .tag_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("tag service not configured"))?;

    let mut result_sets: Vec<HashSet<Uuid>> = Vec::new();

    for tag_name in tags {
        let trimmed = tag_name.trim();
        if trimmed.is_empty() {
            continue;
        }

        let resource_ids = tag_service
            .resource_ids_for_tag_name(trimmed)
            .await
            .map_err(ApiError::from)?;

        let uuid_set = resource_ids
            .into_iter()
            .map(|resource_id| {
                Uuid::parse_str(&resource_id).map_err(|_| {
                    ApiError::from(DomainError::InternalError(format!(
                        "invalid tagged resource id '{resource_id}'"
                    )))
                })
            })
            .collect::<Result<HashSet<_>, _>>()?;

        result_sets.push(uuid_set);
    }

    if result_sets.is_empty() {
        return Ok(None);
    }

    let combined = match logic {
        FilterLogic::And => {
            if let Some(first) = result_sets.pop() {
                let mut result = first;
                for set in result_sets {
                    result = result.intersection(&set).copied().collect();
                }
                result
            } else {
                HashSet::new()
            }
        }
        FilterLogic::Or => {
            let mut result = HashSet::new();
            for set in result_sets {
                result.extend(set);
            }
            result
        }
    };

    if combined.is_empty() {
        Ok(None)
    } else {
        Ok(Some(combined))
    }
}

pub async fn resolve_list_query_filters(
    state: &AppState,
    query: &ListQuery,
) -> Result<Option<HashSet<Uuid>>, ApiError> {
    if !query.tags.is_empty() {
        let logic = if let Some(logic_str) = &query.logic {
            FilterLogic::from_str(logic_str).map_err(ApiError::from)?
        } else {
            FilterLogic::And
        };
        resolve_multi_tag_filter_ids(state, &query.tags, logic).await
    } else {
        resolve_tag_filter_ids(state, query.tag.as_deref()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_logic_from_str_and() {
        let logic = FilterLogic::from_str("and").unwrap();
        assert_eq!(logic, FilterLogic::And);
    }

    #[test]
    fn test_filter_logic_from_str_and_case_insensitive() {
        let logic = FilterLogic::from_str("AND").unwrap();
        assert_eq!(logic, FilterLogic::And);
        let logic = FilterLogic::from_str("And").unwrap();
        assert_eq!(logic, FilterLogic::And);
    }

    #[test]
    fn test_filter_logic_from_str_or() {
        let logic = FilterLogic::from_str("or").unwrap();
        assert_eq!(logic, FilterLogic::Or);
    }

    #[test]
    fn test_filter_logic_from_str_or_case_insensitive() {
        let logic = FilterLogic::from_str("OR").unwrap();
        assert_eq!(logic, FilterLogic::Or);
        let logic = FilterLogic::from_str("Or").unwrap();
        assert_eq!(logic, FilterLogic::Or);
    }

    #[test]
    fn test_filter_logic_from_str_invalid() {
        let result = FilterLogic::from_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_logic_from_str_with_whitespace() {
        let logic = FilterLogic::from_str("  and  ").unwrap();
        assert_eq!(logic, FilterLogic::And);
    }
}
