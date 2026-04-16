use std::sync::Arc;

use async_trait::async_trait;
use domain::{DomainError, Resource, ResourceRepository, SearchStrategy, SearchStrategyKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SearchConfig {
    pub strategy: SearchStrategyKind,
}

pub fn build_search_strategy(config: SearchConfig) -> Arc<dyn SearchStrategy> {
    match config.strategy {
        SearchStrategyKind::Like => Arc::new(LikeSearchStrategy),
        SearchStrategyKind::Fuzzy => Arc::new(FuzzySearchStrategy),
    }
}

#[derive(Debug, Default)]
pub struct LikeSearchStrategy;

#[async_trait]
impl SearchStrategy for LikeSearchStrategy {
    fn kind(&self) -> SearchStrategyKind {
        SearchStrategyKind::Like
    }

    async fn search(
        &self,
        repository: &dyn ResourceRepository,
        query: &str,
    ) -> Result<Vec<Resource>, DomainError> {
        repository.search(query).await
    }
}

#[derive(Debug, Default)]
pub struct FuzzySearchStrategy;

#[async_trait]
impl SearchStrategy for FuzzySearchStrategy {
    fn kind(&self) -> SearchStrategyKind {
        SearchStrategyKind::Fuzzy
    }

    async fn search(
        &self,
        repository: &dyn ResourceRepository,
        query: &str,
    ) -> Result<Vec<Resource>, DomainError> {
        let normalized_query = normalize(query);
        if normalized_query.is_empty() {
            return Err(DomainError::ValidationError(
                "query cannot be empty".to_string(),
            ));
        }

        let mut ranked = Vec::new();
        for resource in repository.list().await? {
            if let Some(score) = fuzzy_score(&resource.title, &normalized_query) {
                ranked.push((score, resource));
            }
        }

        ranked.sort_by(
            |(left_score, left_resource), (right_score, right_resource)| {
                right_score
                    .cmp(left_score)
                    .then_with(|| left_resource.title.cmp(&right_resource.title))
                    .then_with(|| left_resource.id.cmp(&right_resource.id))
            },
        );

        Ok(ranked
            .into_iter()
            .map(|(_, resource)| resource)
            .collect::<Vec<_>>())
    }
}

fn fuzzy_score(title: &str, normalized_query: &str) -> Option<usize> {
    let normalized_title = normalize(title);
    if normalized_title.contains(normalized_query) {
        return Some(1000 + normalized_query.len());
    }

    if is_subsequence(normalized_query, &normalized_title) {
        return Some(normalized_query.len());
    }

    None
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next();

    if current.is_none() {
        return true;
    }

    for character in haystack.chars() {
        if Some(character) == current {
            current = needle_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }

    false
}
