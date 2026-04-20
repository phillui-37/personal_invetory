use std::sync::Arc;

use crate::sqlite;
use domain::{
    ChapterCheckRepository, DomainError, EbookMetaRepository, GameMetaRepository,
    ImageMetaRepository, LocationRepository, NotificationRepository, ResourceRepository,
    VideoMetaRepository, WebReaderMetaRepository,
};

#[cfg(feature = "postgres")]
use crate::postgres;

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
    pub chapter_check_repo: Arc<dyn ChapterCheckRepository>,
    pub notification_repo: Arc<dyn NotificationRepository>,
    pub image_meta_repo: Arc<dyn ImageMetaRepository>,
    pub video_meta_repo: Arc<dyn VideoMetaRepository>,
    pub game_meta_repo: Arc<dyn GameMetaRepository>,
    pub vault_backend: Arc<dyn domain::vault::VaultBackend>,
    pub sync_job_repo: Arc<dyn domain::sync::SyncJobRepository>,
    pub dedup_warning_repo: Arc<dyn domain::dedup::DedupWarningRepository>,
    pub device_repo: Arc<dyn domain::device::DeviceRepository>,
    pub progress_repo: Arc<dyn domain::progress::ProgressRepository>,
    pub tag_repo: Arc<dyn domain::tag::TagRepository>,
    pub resource_tag_repo: Arc<dyn domain::tag::ResourceTagRepository>,
}

pub struct AdapterFactory;

impl AdapterFactory {
    pub async fn from_url(database_url: &str) -> Result<AdapterBundle, DomainError> {
        if database_url.starts_with("sqlite://") {
            let conn = sqlite::open_sqlite_connection(database_url)?;
            let shared = Arc::new(std::sync::Mutex::new(conn));
            let tag_repo = Arc::new(sqlite::tag::SqliteTagRepository::new(shared.clone()));
            return Ok(AdapterBundle {
                database: DatabaseAdapter::Sqlite,
                database_url: database_url.to_string(),
                resource_repo: Arc::new(sqlite::resource::SqliteResourceRepository::new(
                    shared.clone(),
                )),
                ebook_meta_repo: Arc::new(sqlite::ebook_meta::SqliteEbookMetaRepository::new(
                    shared.clone(),
                )),
                web_reader_meta_repo: Arc::new(
                    sqlite::web_reader_meta::SqliteWebReaderMetaRepository::new(shared.clone()),
                ),
                location_repo: Arc::new(sqlite::location::SqliteLocationRepository::new(
                    shared.clone(),
                )),
                chapter_check_repo: Arc::new(
                    sqlite::chapter_check::SqliteChapterCheckRepository::new(shared.clone()),
                ),
                notification_repo: Arc::new(
                    sqlite::notification::SqliteNotificationRepository::new(shared.clone()),
                ),
                image_meta_repo: Arc::new(sqlite::image_meta::SqliteImageMetaRepository::new(
                    shared.clone(),
                )),
                video_meta_repo: Arc::new(sqlite::video_meta::SqliteVideoMetaRepository::new(
                    shared.clone(),
                )),
                game_meta_repo: Arc::new(sqlite::game_meta::SqliteGameMetaRepository::new(
                    shared.clone(),
                )),
                vault_backend: Arc::new(sqlite::vault::SqliteVaultBackend::new(shared.clone())),
                sync_job_repo: Arc::new(sqlite::sync_job::SqliteSyncJobRepository::new(
                    shared.clone(),
                )),
                dedup_warning_repo: Arc::new(sqlite::dedup::SqliteDedupWarningRepository::new(
                    shared.clone(),
                )),
                device_repo: Arc::new(sqlite::device::SqliteDeviceRepository::new(
                    shared.clone(),
                )),
                progress_repo: Arc::new(sqlite::progress::SqliteProgressRepository::new(shared.clone())),
                tag_repo: tag_repo.clone() as Arc<dyn domain::tag::TagRepository>,
                resource_tag_repo: tag_repo as Arc<dyn domain::tag::ResourceTagRepository>,
            });
        }

        #[cfg(feature = "postgres")]
        if database_url.starts_with("postgres://") {
            let pool = postgres::pool::open_pg_pool(database_url).await?;
            postgres::migrations::run_migrations(&pool).await?;
            let tag_repo = Arc::new(postgres::tag::PgTagRepository::new(pool.clone()));
            return Ok(AdapterBundle {
                database: DatabaseAdapter::Postgres,
                database_url: database_url.to_string(),
                resource_repo: Arc::new(postgres::resource::PgResourceRepository::new(pool.clone())),
                ebook_meta_repo: Arc::new(postgres::ebook_meta::PgEbookMetaRepository::new(pool.clone())),
                web_reader_meta_repo: Arc::new(postgres::web_reader_meta::PgWebReaderMetaRepository::new(pool.clone())),
                location_repo: Arc::new(postgres::location::PgLocationRepository::new(pool.clone())),
                chapter_check_repo: Arc::new(postgres::chapter_check::PgChapterCheckRepository::new(pool.clone())),
                notification_repo: Arc::new(postgres::notification::PgNotificationRepository::new(pool.clone())),
                image_meta_repo: Arc::new(postgres::image_meta::PgImageMetaRepository::new(pool.clone())),
                video_meta_repo: Arc::new(postgres::video_meta::PgVideoMetaRepository::new(pool.clone())),
                game_meta_repo: Arc::new(postgres::game_meta::PgGameMetaRepository::new(pool.clone())),
                vault_backend: Arc::new(postgres::vault::PgVaultBackend::new(pool.clone())),
                sync_job_repo: Arc::new(postgres::sync_job::PgSyncJobRepository::new(pool.clone())),
                dedup_warning_repo: Arc::new(postgres::dedup::PgDedupWarningRepository::new(pool.clone())),
                device_repo: Arc::new(postgres::device::PgDeviceRepository::new(pool.clone())),
                progress_repo: Arc::new(postgres::progress::PgProgressRepository::new(pool.clone())),
                tag_repo: tag_repo.clone() as Arc<dyn domain::tag::TagRepository>,
                resource_tag_repo: tag_repo as Arc<dyn domain::tag::ResourceTagRepository>,
            });
        }

        #[cfg(not(feature = "postgres"))]
        if database_url.starts_with("postgres://") {
            return Err(DomainError::ValidationError(
                "Postgres support not compiled in; rebuild with --features postgres".to_string(),
            ));
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
