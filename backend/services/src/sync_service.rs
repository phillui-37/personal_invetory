use std::sync::Arc;

use domain::sync::{NewSyncJob, SyncJob, SyncJobRepository, SyncJobStatus};
use domain::{DomainError, GameMetaRepository, NewGameMeta, NewResource, ResourceRepository, ResourceType};
use plugins::ecosystem::steam::SteamOwnedGame;
use uuid::Uuid;

pub struct SyncService {
    resource_repo: Arc<dyn ResourceRepository>,
    game_meta_repo: Arc<dyn GameMetaRepository>,
    sync_job_repo: Arc<dyn SyncJobRepository>,
}

impl SyncService {
    pub fn new(
        resource_repo: Arc<dyn ResourceRepository>,
        game_meta_repo: Arc<dyn GameMetaRepository>,
        sync_job_repo: Arc<dyn SyncJobRepository>,
    ) -> Self {
        Self {
            resource_repo,
            game_meta_repo,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use domain::{GameMeta, Resource, UpdateResource};
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
        let svc = SyncService::new(
            Arc::new(MockResourceRepo::new()),
            Arc::new(MockGameMetaRepo),
            Arc::new(MockSyncJobRepo::new()),
        );
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
}
