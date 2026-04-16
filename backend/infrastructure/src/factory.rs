use std::sync::Arc;

use crate::{postgres, sqlite};
use domain::{
    DomainError, EbookMetaRepository, LocationRepository, ResourceRepository, WebReaderMetaRepository,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseAdapter {
    Postgres,
    Sqlite,
}

pub struct AdapterBundle {
    pub database: DatabaseAdapter,
    pub database_url: String,
    pub resource_repo: Arc<dyn ResourceRepository>,
    pub ebook_meta_repo: Arc<dyn EbookMetaRepository>,
    pub web_reader_meta_repo: Arc<dyn WebReaderMetaRepository>,
    pub location_repo: Arc<dyn LocationRepository>,
}

pub struct AdapterFactory;

impl AdapterFactory {
    pub fn from_url(database_url: &str) -> Result<AdapterBundle, DomainError> {
        if database_url.starts_with("sqlite://") {
            let conn = sqlite::open_sqlite_connection(database_url)?;
            let shared = Arc::new(std::sync::Mutex::new(conn));
            return Ok(AdapterBundle {
                database: DatabaseAdapter::Sqlite,
                database_url: database_url.to_string(),
                resource_repo: Arc::new(sqlite::resource::SqliteResourceRepository::new(shared.clone())),
                ebook_meta_repo: Arc::new(sqlite::ebook_meta::SqliteEbookMetaRepository::new(shared.clone())),
                web_reader_meta_repo: Arc::new(sqlite::web_reader_meta::SqliteWebReaderMetaRepository::new(shared.clone())),
                location_repo: Arc::new(sqlite::location::SqliteLocationRepository::new(shared)),
            });
        }

        if database_url.starts_with("postgres://") {
            return Ok(AdapterBundle {
                database: DatabaseAdapter::Postgres,
                database_url: database_url.to_string(),
                resource_repo: Arc::new(postgres::resource::PostgresResourceRepository),
                ebook_meta_repo: Arc::new(postgres::ebook_meta::PostgresEbookMetaRepository),
                web_reader_meta_repo: Arc::new(postgres::web_reader_meta::PostgresWebReaderMetaRepository),
                location_repo: Arc::new(postgres::location::PostgresLocationRepository),
            });
        }

        Err(DomainError::ValidationError(format!(
            "Unsupported database URL prefix in '{database_url}'. Expected 'postgres://' or 'sqlite://'"
        )))
    }
}

pub fn resolve_search_strategy(
    _database: &DatabaseAdapter,
    configured: Option<&str>,
) -> domain::SearchStrategyKind {
    domain::SearchStrategyKind::from_config(configured)
}
