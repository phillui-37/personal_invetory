use std::sync::Arc;

use domain::ecosystem::DiscoveredItem;
use domain::sync::{NewSyncJob, SyncJob, SyncJobRepository, SyncJobStatus};
use domain::{
    DomainError, EbookMetaRepository, GameMetaRepository, ImageMetaRepository,
    NewEbookMeta, NewGameMeta, NewImageMeta, NewResource, NewVideoMeta,
    ResourceRepository, ResourceType, VideoMetaRepository,
};
use plugins::ecosystem::steam::SteamOwnedGame;
use uuid::Uuid;

fn parse_resource_type(value: Option<&String>) -> ResourceType {
    match value.map(|s| s.as_str()) {
        Some("Ebook") => ResourceType::Ebook,
        Some("Image") => ResourceType::Image,
        Some("Video") => ResourceType::Video,
        Some("WebReader") => ResourceType::WebReader,
        _ => ResourceType::Game,
    }
}

pub struct SyncService {
    resource_repo: Arc<dyn ResourceRepository>,
    game_meta_repo: Arc<dyn GameMetaRepository>,
    ebook_meta_repo: Arc<dyn EbookMetaRepository>,
    image_meta_repo: Arc<dyn ImageMetaRepository>,
    video_meta_repo: Arc<dyn VideoMetaRepository>,
    sync_job_repo: Arc<dyn SyncJobRepository>,
}

impl SyncService {
    pub fn new(
        resource_repo: Arc<dyn ResourceRepository>,
        game_meta_repo: Arc<dyn GameMetaRepository>,
        ebook_meta_repo: Arc<dyn EbookMetaRepository>,
        image_meta_repo: Arc<dyn ImageMetaRepository>,
        video_meta_repo: Arc<dyn VideoMetaRepository>,
        sync_job_repo: Arc<dyn SyncJobRepository>,
    ) -> Self {
        Self {
            resource_repo,
            game_meta_repo,
            ebook_meta_repo,
            image_meta_repo,
            video_meta_repo,
            sync_job_repo,
        }
    }

    pub async fn sync_steam_games(
        &self,
        games: Vec<SteamOwnedGame>,
    ) -> Result<SyncJob, DomainError> {
        let mut job = self
            .sync_job_repo
            .create(NewSyncJob {
                platform: "steam".to_string(),
            })
            .await?;

        job.status = SyncJobStatus::Running;
        self.sync_job_repo.update(&job).await?;

        let result = self.run_steam_sync(&mut job, &games).await;

        match result {
            Ok(()) => {
                job.status = SyncJobStatus::Completed;
                self.sync_job_repo.update(&job).await?;
                Ok(job)
            }
            Err(e) => {
                job.status = SyncJobStatus::Failed;
                job.error_message = Some(format!("{e:?}"));
                let _ = self.sync_job_repo.update(&job).await;
                Err(e)
            }
        }
    }

    async fn run_steam_sync(
        &self,
        job: &mut SyncJob,
        games: &[SteamOwnedGame],
    ) -> Result<(), DomainError> {
        let existing = self.resource_repo.list().await?;
        let existing_titles: Vec<String> = existing
            .iter()
            .filter(|r| r.resource_type == ResourceType::Game)
            .map(|r| r.title.to_lowercase())
            .collect();

        job.items_found = games.len() as u32;
        let mut created = 0u32;
        let mut skipped = 0u32;

        for game in games {
            let title_lower = game.name.to_lowercase();
            if existing_titles.contains(&title_lower) {
                skipped += 1;
                continue;
            }

            let resource = self
                .resource_repo
                .create(NewResource {
                    title: game.name.clone(),
                    notes: None,
                    resource_type: ResourceType::Game,
                })
                .await?;

            let playtime_note = game
                .playtime_forever
                .map(|m| format!("Steam playtime: {} hours", m / 60));

            self.game_meta_repo
                .upsert(
                    resource.id,
                    NewGameMeta {
                        platform: Some("PC".to_string()),
                        store: Some("Steam".to_string()),
                        developer: None,
                        publisher: None,
                        manual_notes: playtime_note,
                    },
                )
                .await?;

            created += 1;
        }

        job.items_created = created;
        job.items_skipped = skipped;
        job.items_failed = 0;
        Ok(())
    }

    pub async fn list_jobs(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError> {
        self.sync_job_repo.list_by_platform(platform).await
    }

    pub async fn get_job(&self, id: Uuid) -> Result<SyncJob, DomainError> {
        self.sync_job_repo.get(id).await
    }

    /// Generic multi-platform sync: creates resources from already-fetched `DiscoveredItem`s.
    ///
    /// The `resource_type` override in `metadata["resource_type"]` is parsed from Debug format
    /// (e.g. "Game", "Ebook", "Image", "Video"). Defaults to Game if absent or unrecognised.
    pub async fn sync_discovered_items(
        &self,
        platform: &str,
        items: Vec<DiscoveredItem>,
    ) -> Result<SyncJob, DomainError> {
        let mut job = self
            .sync_job_repo
            .create(NewSyncJob {
                platform: platform.to_string(),
            })
            .await?;

        job.status = SyncJobStatus::Running;
        self.sync_job_repo.update(&job).await?;

        match self.run_discovered_sync(&mut job, items).await {
            Ok(()) => {
                job.status = SyncJobStatus::Completed;
                self.sync_job_repo.update(&job).await?;
                Ok(job)
            }
            Err(e) => {
                job.status = SyncJobStatus::Failed;
                job.error_message = Some(format!("{e:?}"));
                let _ = self.sync_job_repo.update(&job).await;
                Err(e)
            }
        }
    }

    async fn run_discovered_sync(
        &self,
        job: &mut SyncJob,
        items: Vec<DiscoveredItem>,
    ) -> Result<(), DomainError> {
        let existing = self.resource_repo.list().await?;
        let mut seen_titles: std::collections::HashSet<String> =
            existing.iter().map(|r| r.title.to_lowercase()).collect();

        job.items_found = items.len() as u32;
        let mut created = 0u32;
        let mut skipped = 0u32;
        let mut failed = 0u32;

        for item in items {
            let lower = item.title.to_lowercase();
            if seen_titles.contains(&lower) {
                skipped += 1;
                continue;
            }

            let resource_type = parse_resource_type(item.metadata.get("resource_type"));
            let resource = match self
                .resource_repo
                .create(NewResource {
                    title: item.title.clone(),
                    notes: item.metadata.get("notes").cloned(),
                    resource_type: resource_type.clone(),
                })
                .await
            {
                Ok(r) => r,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };

            let meta_result = match resource_type {
                ResourceType::Ebook => {
                    self.ebook_meta_repo
                        .upsert(
                            resource.id,
                            NewEbookMeta {
                                author: item.metadata.get("author").cloned(),
                                isbn: None,
                                publisher: item.metadata.get("publisher").cloned(),
                                language: None,
                                file_format: item.metadata.get("file_format").cloned(),
                            },
                        )
                        .await
                        .map(|_| ())
                }
                ResourceType::Image => {
                    self.image_meta_repo
                        .upsert(
                            resource.id,
                            NewImageMeta {
                                width: None,
                                height: None,
                                file_format: item.metadata.get("file_format").cloned(),
                                file_size_bytes: None,
                            },
                        )
                        .await
                        .map(|_| ())
                }
                ResourceType::Video => {
                    self.video_meta_repo
                        .upsert(
                            resource.id,
                            NewVideoMeta {
                                duration_secs: None,
                                file_format: item.metadata.get("file_format").cloned(),
                                resolution: None,
                                file_size_bytes: None,
                            },
                        )
                        .await
                        .map(|_| ())
                }
                ResourceType::Game => {
                    self.game_meta_repo
                        .upsert(
                            resource.id,
                            NewGameMeta {
                                platform: None,
                                store: Some(item.platform.clone()),
                                developer: item.metadata.get("maker").cloned(),
                                publisher: None,
                                manual_notes: None,
                            },
                        )
                        .await
                        .map(|_| ())
                }
                // WebReader is URL-based; no connector-discoverable metadata applies.
                ResourceType::WebReader => Ok(()),
            };

            match meta_result {
                Ok(()) => {
                    seen_titles.insert(lower);
                    created += 1;
                }
                Err(_) => {
                    // Roll back the orphaned resource to keep DB consistent.
                    let _ = self.resource_repo.delete(resource.id).await;
                    failed += 1;
                }
            }
        }

        job.items_created = created;
        job.items_skipped = skipped;
        job.items_failed = failed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use domain::{
        EbookMetaRepository, GameMeta, ImageMetaRepository, Resource, UpdateResource,
        VideoMetaRepository,
    };
    use std::sync::Mutex;

    struct MockResourceRepo {
        resources: Mutex<Vec<Resource>>,
    }
    impl MockResourceRepo {
        fn new() -> Self {
            Self {
                resources: Mutex::new(Vec::new()),
            }
        }
    }
    #[async_trait]
    impl ResourceRepository for MockResourceRepo {
        async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
            let r = Resource {
                id: Uuid::new_v4(),
                title: input.title,
                notes: input.notes,
                resource_type: input.resource_type,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.resources.lock().unwrap().push(r.clone());
            Ok(r)
        }
        async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
            self.resources
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.id == id)
                .cloned()
                .ok_or(DomainError::NotFound("not found".into()))
        }
        async fn list(&self) -> Result<Vec<Resource>, DomainError> {
            Ok(self.resources.lock().unwrap().clone())
        }
        async fn update(
            &self,
            _id: Uuid,
            _input: UpdateResource,
        ) -> Result<Resource, DomainError> {
            Err(DomainError::NotFound("not implemented".into()))
        }
        async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }
        async fn search(&self, _query: &str) -> Result<Vec<Resource>, DomainError> {
            Ok(vec![])
        }
    }

    struct MockGameMetaRepo;
    #[async_trait]
    impl GameMetaRepository for MockGameMetaRepo {
        async fn get(&self, _resource_id: Uuid) -> Result<GameMeta, DomainError> {
            Err(DomainError::NotFound("not found".into()))
        }
        async fn upsert(
            &self,
            resource_id: Uuid,
            input: NewGameMeta,
        ) -> Result<GameMeta, DomainError> {
            Ok(GameMeta {
                resource_id,
                platform: input.platform,
                store: input.store,
                developer: input.developer,
                publisher: input.publisher,
                manual_notes: input.manual_notes,
            })
        }
    }

    struct MockEbookMetaRepo;
    #[async_trait]
    impl EbookMetaRepository for MockEbookMetaRepo {
        async fn get(&self, resource_id: Uuid) -> Result<domain::EbookMeta, DomainError> {
            Err(DomainError::NotFound(format!("ebook meta {resource_id} not found")))
        }
        async fn upsert(
            &self,
            resource_id: Uuid,
            input: NewEbookMeta,
        ) -> Result<domain::EbookMeta, DomainError> {
            Ok(domain::EbookMeta {
                resource_id,
                author: input.author,
                isbn: input.isbn,
                publisher: input.publisher,
                language: input.language,
                file_format: input.file_format,
            })
        }
    }

    struct MockImageMetaRepo;
    #[async_trait]
    impl ImageMetaRepository for MockImageMetaRepo {
        async fn get(&self, resource_id: Uuid) -> Result<domain::ImageMeta, DomainError> {
            Err(DomainError::NotFound(format!("image meta {resource_id} not found")))
        }
        async fn upsert(
            &self,
            resource_id: Uuid,
            input: NewImageMeta,
        ) -> Result<domain::ImageMeta, DomainError> {
            Ok(domain::ImageMeta {
                resource_id,
                width: input.width,
                height: input.height,
                file_format: input.file_format,
                file_size_bytes: input.file_size_bytes,
            })
        }
    }

    struct MockVideoMetaRepo;
    #[async_trait]
    impl VideoMetaRepository for MockVideoMetaRepo {
        async fn get(&self, resource_id: Uuid) -> Result<domain::VideoMeta, DomainError> {
            Err(DomainError::NotFound(format!("video meta {resource_id} not found")))
        }
        async fn upsert(
            &self,
            resource_id: Uuid,
            input: NewVideoMeta,
        ) -> Result<domain::VideoMeta, DomainError> {
            Ok(domain::VideoMeta {
                resource_id,
                duration_secs: input.duration_secs,
                file_format: input.file_format,
                resolution: input.resolution,
                file_size_bytes: input.file_size_bytes,
            })
        }
    }

    struct MockSyncJobRepo {
        jobs: Mutex<Vec<SyncJob>>,
    }
    impl MockSyncJobRepo {
        fn new() -> Self {
            Self {
                jobs: Mutex::new(Vec::new()),
            }
        }
    }
    #[async_trait]
    impl SyncJobRepository for MockSyncJobRepo {
        async fn create(&self, input: NewSyncJob) -> Result<SyncJob, DomainError> {
            let job = SyncJob {
                id: Uuid::new_v4(),
                platform: input.platform,
                status: SyncJobStatus::Pending,
                started_at: None,
                completed_at: None,
                items_found: 0,
                items_created: 0,
                items_skipped: 0,
                items_failed: 0,
                error_message: None,
                created_at: Utc::now(),
            };
            self.jobs.lock().unwrap().push(job.clone());
            Ok(job)
        }
        async fn update(&self, job: &SyncJob) -> Result<(), DomainError> {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(j) = jobs.iter_mut().find(|j| j.id == job.id) {
                *j = job.clone();
            }
            Ok(())
        }
        async fn get(&self, id: Uuid) -> Result<SyncJob, DomainError> {
            self.jobs
                .lock()
                .unwrap()
                .iter()
                .find(|j| j.id == id)
                .cloned()
                .ok_or(DomainError::NotFound("not found".into()))
        }
        async fn list_by_platform(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .filter(|j| j.platform == platform)
                .cloned()
                .collect())
        }
    }

    fn make_service() -> SyncService {
        SyncService::new(
            Arc::new(MockResourceRepo::new()),
            Arc::new(MockGameMetaRepo),
            Arc::new(MockEbookMetaRepo),
            Arc::new(MockImageMetaRepo),
            Arc::new(MockVideoMetaRepo),
            Arc::new(MockSyncJobRepo::new()),
        )
    }

    #[test]
    fn sync_steam_creates_resources_and_job() {
        let svc = make_service();
        futures::executor::block_on(async {
            let games = vec![
                SteamOwnedGame {
                    appid: 440,
                    name: "Team Fortress 2".into(),
                    playtime_forever: Some(6000),
                },
                SteamOwnedGame {
                    appid: 570,
                    name: "Dota 2".into(),
                    playtime_forever: Some(120),
                },
            ];
            let job = svc.sync_steam_games(games).await.unwrap();
            assert_eq!(job.status, SyncJobStatus::Completed);
            assert_eq!(job.items_found, 2);
            assert_eq!(job.items_created, 2);
            assert_eq!(job.items_skipped, 0);
        });
    }

    #[test]
    fn sync_steam_skips_existing_games() {
        let svc = make_service();
        futures::executor::block_on(async {
            let games = vec![SteamOwnedGame {
                appid: 440,
                name: "Team Fortress 2".into(),
                playtime_forever: None,
            }];
            svc.sync_steam_games(games).await.unwrap();

            let games2 = vec![
                SteamOwnedGame {
                    appid: 440,
                    name: "Team Fortress 2".into(),
                    playtime_forever: None,
                },
                SteamOwnedGame {
                    appid: 570,
                    name: "Dota 2".into(),
                    playtime_forever: None,
                },
            ];
            let job = svc.sync_steam_games(games2).await.unwrap();
            assert_eq!(job.items_found, 2);
            assert_eq!(job.items_created, 1);
            assert_eq!(job.items_skipped, 1);
        });
    }

    #[test]
    fn list_jobs_returns_platform_jobs() {
        let svc = make_service();
        futures::executor::block_on(async {
            svc.sync_steam_games(vec![SteamOwnedGame {
                appid: 1,
                name: "Game".into(),
                playtime_forever: None,
            }])
            .await
            .unwrap();
            let jobs = svc.list_jobs("steam").await.unwrap();
            assert_eq!(jobs.len(), 1);
            assert_eq!(jobs[0].platform, "steam");
        });
    }

    #[test]
    fn parse_resource_type_parses_known_types() {
        let game = "Game".to_string();
        let ebook = "Ebook".to_string();
        let image = "Image".to_string();
        let video = "Video".to_string();
        assert_eq!(super::parse_resource_type(Some(&game)), ResourceType::Game);
        assert_eq!(super::parse_resource_type(Some(&ebook)), ResourceType::Ebook);
        assert_eq!(super::parse_resource_type(Some(&image)), ResourceType::Image);
        assert_eq!(super::parse_resource_type(Some(&video)), ResourceType::Video);
        assert_eq!(super::parse_resource_type(None), ResourceType::Game);
        let unknown = "Unknown".to_string();
        assert_eq!(super::parse_resource_type(Some(&unknown)), ResourceType::Game);
    }

    #[test]
    fn sync_discovered_items_creates_game_resources() {
        let svc = make_service();
        futures::executor::block_on(async {
            let mut meta = std::collections::HashMap::new();
            meta.insert("resource_type".to_string(), "Game".to_string());
            meta.insert("maker".to_string(), "CircleX".to_string());
            let items = vec![
                DiscoveredItem {
                    external_id: "RJ001".to_string(),
                    title: "DLSite Game 1".to_string(),
                    platform: "dlsite".to_string(),
                    metadata: meta.clone(),
                },
                DiscoveredItem {
                    external_id: "RJ002".to_string(),
                    title: "DLSite Game 2".to_string(),
                    platform: "dlsite".to_string(),
                    metadata: meta,
                },
            ];
            let job = svc.sync_discovered_items("dlsite", items).await.unwrap();
            assert_eq!(job.status, SyncJobStatus::Completed);
            assert_eq!(job.items_found, 2);
            assert_eq!(job.items_created, 2);
            assert_eq!(job.items_skipped, 0);
        });
    }

    #[test]
    fn sync_discovered_items_creates_ebook_resources() {
        let svc = make_service();
        futures::executor::block_on(async {
            let mut meta = std::collections::HashMap::new();
            meta.insert("resource_type".to_string(), "Ebook".to_string());
            meta.insert("author".to_string(), "Auth X".to_string());
            meta.insert("file_format".to_string(), "bookwalker_digital".to_string());
            let items = vec![DiscoveredItem {
                external_id: "bw-001".to_string(),
                title: "BW Book".to_string(),
                platform: "bookwalker".to_string(),
                metadata: meta,
            }];
            let job = svc.sync_discovered_items("bookwalker", items).await.unwrap();
            assert_eq!(job.status, SyncJobStatus::Completed);
            assert_eq!(job.items_created, 1);
        });
    }

    #[test]
    fn sync_discovered_items_skips_existing_titles() {
        let svc = make_service();
        futures::executor::block_on(async {
            let mut meta = std::collections::HashMap::new();
            meta.insert("resource_type".to_string(), "Game".to_string());
            let items = vec![DiscoveredItem {
                external_id: "x1".to_string(),
                title: "Existing Game".to_string(),
                platform: "dlsite".to_string(),
                metadata: meta.clone(),
            }];
            svc.sync_discovered_items("dlsite", items).await.unwrap();

            // Sync same title again
            let items2 = vec![
                DiscoveredItem {
                    external_id: "x1".to_string(),
                    title: "Existing Game".to_string(),
                    platform: "dlsite".to_string(),
                    metadata: meta.clone(),
                },
                DiscoveredItem {
                    external_id: "x2".to_string(),
                    title: "New Game".to_string(),
                    platform: "dlsite".to_string(),
                    metadata: meta,
                },
            ];
            let job = svc.sync_discovered_items("dlsite", items2).await.unwrap();
            assert_eq!(job.items_found, 2);
            assert_eq!(job.items_created, 1);
            assert_eq!(job.items_skipped, 1);
        });
    }
}
