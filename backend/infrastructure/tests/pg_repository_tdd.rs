//! TDD tests for PostgreSQL repository implementations.
//! Tests are skipped when `TEST_PG_URL` environment variable is not set.

#[cfg(feature = "postgres")]
mod pg_tests {
    use domain::{
        DomainError, NewEbookMeta, NewResource, NewResourceLocation, NewWebReaderMeta,
        ResourceType, StorageType, UpdateResource,
    };
    use infrastructure::postgres::{migrations, pool};

    async fn setup() -> Option<sqlx::PgPool> {
        let url = std::env::var("TEST_PG_URL").ok()?;
        let p = pool::open_pg_pool(&url).await.expect("open_pg_pool");
        migrations::run_migrations(&p).await.expect("run_migrations");
        Some(p)
    }

    // ── Resource ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_resource_create_and_list() {
        let Some(pool) = setup().await else { return };
        let repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        use domain::ResourceRepository;

        let r = repo
            .create(NewResource {
                title: format!("pg_test_create_{}", uuid::Uuid::new_v4()),
                notes: Some("note".to_string()),
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create");

        let list = repo.list().await.expect("list");
        assert!(list.iter().any(|x| x.id == r.id));

        // cleanup
        repo.delete(r.id).await.expect("delete");
    }

    #[tokio::test]
    async fn pg_resource_get_by_id_not_found() {
        let Some(pool) = setup().await else { return };
        let repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        use domain::ResourceRepository;

        let result = repo.get_by_id(uuid::Uuid::new_v4()).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }

    #[tokio::test]
    async fn pg_resource_update_title_conflict() {
        let Some(pool) = setup().await else { return };
        let repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        use domain::ResourceRepository;

        let base = format!("pg_conflict_base_{}", uuid::Uuid::new_v4());
        let other = format!("pg_conflict_other_{}", uuid::Uuid::new_v4());
        let r1 = repo
            .create(NewResource {
                title: base.clone(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create r1");
        let r2 = repo
            .create(NewResource {
                title: other.clone(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create r2");

        let result = repo
            .update(
                r2.id,
                UpdateResource {
                    title: Some(base.clone()),
                    notes: None,
                },
            )
            .await;
        assert!(matches!(result, Err(DomainError::Conflict(_))));

        // cleanup
        repo.delete(r1.id).await.ok();
        repo.delete(r2.id).await.ok();
    }

    #[tokio::test]
    async fn pg_resource_delete() {
        let Some(pool) = setup().await else { return };
        let repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        use domain::ResourceRepository;

        let r = repo
            .create(NewResource {
                title: format!("pg_delete_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .expect("create");

        repo.delete(r.id).await.expect("delete");
        let result = repo.get_by_id(r.id).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }

    // ── EbookMeta ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_ebook_meta_upsert_and_get() {
        let Some(pool) = setup().await else { return };
        let res_repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        let meta_repo =
            infrastructure::postgres::ebook_meta::PgEbookMetaRepository::new(pool.clone());
        use domain::{EbookMetaRepository, ResourceRepository};

        let r = res_repo
            .create(NewResource {
                title: format!("pg_ebook_meta_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create resource");

        let meta = meta_repo
            .upsert(
                r.id,
                NewEbookMeta {
                    author: Some("Author".to_string()),
                    isbn: Some("978-0000000000".to_string()),
                    publisher: Some("Pub".to_string()),
                    language: Some("en".to_string()),
                    file_format: Some("epub".to_string()),
                },
            )
            .await
            .expect("upsert");

        assert_eq!(meta.resource_id, r.id);
        assert_eq!(meta.author, Some("Author".to_string()));

        let fetched = meta_repo.get(r.id).await.expect("get");
        assert_eq!(fetched.isbn, Some("978-0000000000".to_string()));

        // cleanup
        res_repo.delete(r.id).await.ok();
    }

    // ── WebReaderMeta ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_web_reader_meta_upsert_and_get() {
        let Some(pool) = setup().await else { return };
        let res_repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        let meta_repo =
            infrastructure::postgres::web_reader_meta::PgWebReaderMetaRepository::new(pool.clone());
        use domain::{ResourceRepository, WebReaderMetaRepository};

        let r = res_repo
            .create(NewResource {
                title: format!("pg_web_meta_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::WebReader,
            })
            .await
            .expect("create resource");

        let meta = meta_repo
            .upsert(
                r.id,
                NewWebReaderMeta {
                    url: "https://example.com/novel".to_string(),
                    site_name: Some("Example".to_string()),
                    last_checked_chapter: Some("ch1".to_string()),
                    check_interval_secs: Some(3600),
                    last_checked_at: None,
                    progress_css_selector: Some(".chapter".to_string()),
                },
            )
            .await
            .expect("upsert");

        assert_eq!(meta.resource_id, r.id);
        assert_eq!(meta.check_interval_secs, Some(3600));

        let fetched = meta_repo.get(r.id).await.expect("get");
        assert_eq!(fetched.url, "https://example.com/novel");
        assert_eq!(fetched.site_name, Some("Example".to_string()));

        // cleanup
        res_repo.delete(r.id).await.ok();
    }

    // ── Location ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_location_add_list_remove() {
        let Some(pool) = setup().await else { return };
        let res_repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        let loc_repo =
            infrastructure::postgres::location::PgLocationRepository::new(pool.clone());
        use domain::{LocationRepository, ResourceRepository};

        let r = res_repo
            .create(NewResource {
                title: format!("pg_location_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::Image,
            })
            .await
            .expect("create resource");

        let loc = loc_repo
            .add(
                r.id,
                NewResourceLocation {
                    device_id: "dev-001".to_string(),
                    path_or_url: "/data/image.png".to_string(),
                    storage_type: StorageType::LocalFs,
                },
            )
            .await
            .expect("add");

        let list = loc_repo.list(r.id).await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, loc.id);

        loc_repo.remove(r.id, loc.id).await.expect("remove");

        let list2 = loc_repo.list(r.id).await.expect("list after remove");
        assert!(list2.is_empty());

        // cleanup
        res_repo.delete(r.id).await.ok();
    }

    // ── ChapterCheck ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_chapter_check_create_and_list() {
        let Some(pool) = setup().await else { return };
        let res_repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        let cc_repo =
            infrastructure::postgres::chapter_check::PgChapterCheckRepository::new(pool.clone());
        use domain::{ChapterCheckRepository, ResourceRepository};

        let r = res_repo
            .create(NewResource {
                title: format!("pg_cc_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::WebReader,
            })
            .await
            .expect("create resource");

        let cc = cc_repo
            .create(r.id, true, Some("Chapter 42".to_string()), None)
            .await
            .expect("create chapter check");

        assert_eq!(cc.resource_id, r.id);
        assert!(cc.has_new_chapter);
        assert_eq!(cc.latest_chapter, Some("Chapter 42".to_string()));

        let list = cc_repo.list(r.id).await.expect("list");
        assert!(list.iter().any(|x| x.id == cc.id));

        // cleanup
        res_repo.delete(r.id).await.ok();
    }

    // ── Notification ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_notification_create_list_mark_read() {
        let Some(pool) = setup().await else { return };
        let res_repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        let notif_repo =
            infrastructure::postgres::notification::PgNotificationRepository::new(pool.clone());
        use domain::{NotificationRepository, ResourceRepository};

        let r = res_repo
            .create(NewResource {
                title: format!("pg_notif_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::WebReader,
            })
            .await
            .expect("create resource");

        let n = notif_repo
            .create(r.id, "new chapter available".to_string())
            .await
            .expect("create notification");

        assert_eq!(n.resource_id, r.id);
        assert!(!n.read);

        let unread = notif_repo.list(true).await.expect("list unread");
        assert!(unread.iter().any(|x| x.id == n.id));

        notif_repo.mark_read(n.id).await.expect("mark_read");

        let unread2 = notif_repo.list(true).await.expect("list unread after mark");
        assert!(!unread2.iter().any(|x| x.id == n.id));

        // cleanup
        res_repo.delete(r.id).await.ok();
    }

    // ── ImageMeta ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_image_meta_upsert_and_get() {
        let Some(pool) = setup().await else { return };
        let res_repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        let meta_repo =
            infrastructure::postgres::image_meta::PgImageMetaRepository::new(pool.clone());
        use domain::{ImageMetaRepository, NewImageMeta, ResourceRepository};

        let r = res_repo
            .create(NewResource {
                title: format!("pg_img_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::Image,
            })
            .await
            .expect("create resource");

        let meta = meta_repo
            .upsert(
                r.id,
                NewImageMeta {
                    width: Some(1920),
                    height: Some(1080),
                    file_format: Some("jpeg".to_string()),
                    file_size_bytes: Some(204800),
                },
            )
            .await
            .expect("upsert");

        assert_eq!(meta.resource_id, r.id);
        assert_eq!(meta.width, Some(1920));
        assert_eq!(meta.height, Some(1080));

        let fetched = meta_repo.get(r.id).await.expect("get");
        assert_eq!(fetched.file_format, Some("jpeg".to_string()));
        assert_eq!(fetched.file_size_bytes, Some(204800));

        // cleanup
        res_repo.delete(r.id).await.ok();
    }

    // ── VideoMeta ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_video_meta_upsert_and_get() {
        let Some(pool) = setup().await else { return };
        let res_repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        let meta_repo =
            infrastructure::postgres::video_meta::PgVideoMetaRepository::new(pool.clone());
        use domain::{NewVideoMeta, ResourceRepository, VideoMetaRepository};

        let r = res_repo
            .create(NewResource {
                title: format!("pg_vid_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::Video,
            })
            .await
            .expect("create resource");

        let meta = meta_repo
            .upsert(
                r.id,
                NewVideoMeta {
                    duration_secs: Some(3600),
                    file_format: Some("mp4".to_string()),
                    resolution: Some("1920x1080".to_string()),
                    file_size_bytes: Some(1073741824),
                },
            )
            .await
            .expect("upsert");

        assert_eq!(meta.resource_id, r.id);
        assert_eq!(meta.duration_secs, Some(3600));

        let fetched = meta_repo.get(r.id).await.expect("get");
        assert_eq!(fetched.resolution, Some("1920x1080".to_string()));
        assert_eq!(fetched.file_format, Some("mp4".to_string()));

        // cleanup
        res_repo.delete(r.id).await.ok();
    }

    // ── GameMeta ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pg_game_meta_upsert_and_get() {
        let Some(pool) = setup().await else { return };
        let res_repo = infrastructure::postgres::resource::PgResourceRepository::new(pool.clone());
        let meta_repo =
            infrastructure::postgres::game_meta::PgGameMetaRepository::new(pool.clone());
        use domain::{GameMetaRepository, NewGameMeta, ResourceRepository};

        let r = res_repo
            .create(NewResource {
                title: format!("pg_game_{}", uuid::Uuid::new_v4()),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .expect("create resource");

        let meta = meta_repo
            .upsert(
                r.id,
                NewGameMeta {
                    platform: Some("PC".to_string()),
                    store: Some("Steam".to_string()),
                    developer: Some("Dev Studio".to_string()),
                    publisher: Some("Pub Co".to_string()),
                    manual_notes: Some("Great game".to_string()),
                },
            )
            .await
            .expect("upsert");

        assert_eq!(meta.resource_id, r.id);
        assert_eq!(meta.platform, Some("PC".to_string()));
        assert_eq!(meta.store, Some("Steam".to_string()));

        let fetched = meta_repo.get(r.id).await.expect("get");
        assert_eq!(fetched.developer, Some("Dev Studio".to_string()));
        assert_eq!(fetched.manual_notes, Some("Great game".to_string()));

        // cleanup
        res_repo.delete(r.id).await.ok();
    }
}

// ── P7-D: vault, sync_job, dedup, device, progress, tag PG repos ───────────

#[cfg(feature = "postgres")]
#[tokio::test]
async fn pg_vault_backend_config_roundtrip() {
    let url = match std::env::var("TEST_PG_URL") {
        Ok(u) => u,
        Err(_) => return,
    };
    let pool = infrastructure::postgres::pool::open_pg_pool(&url).await.unwrap();
    infrastructure::postgres::migrations::run_migrations(&pool).await.unwrap();

    use infrastructure::postgres::vault::PgVaultBackend;
    use domain::vault::VaultBackend;

    let backend = PgVaultBackend::new(pool);
    let cfg = domain::vault::VaultConfig {
        salt: vec![1, 2, 3, 4],
        key_check: vec![5, 6, 7, 8],
        key_check_nonce: vec![9, 10],
    };
    backend.save_config(&cfg).await.expect("save_config");
    let loaded = backend.get_config().await.expect("get_config").expect("should exist");
    assert_eq!(loaded.salt, cfg.salt);
    assert_eq!(loaded.key_check, cfg.key_check);
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn pg_sync_job_repository_create_and_get() {
    let url = match std::env::var("TEST_PG_URL") {
        Ok(u) => u,
        Err(_) => return,
    };
    let pool = infrastructure::postgres::pool::open_pg_pool(&url).await.unwrap();
    infrastructure::postgres::migrations::run_migrations(&pool).await.unwrap();

    use infrastructure::postgres::sync_job::PgSyncJobRepository;
    use domain::sync::{NewSyncJob, SyncJobRepository, SyncJobStatus};

    let repo = PgSyncJobRepository::new(pool);
    let job = repo.create(NewSyncJob { platform: "test_platform".to_string() })
        .await.expect("create");

    assert_eq!(job.platform, "test_platform");
    assert!(matches!(job.status, SyncJobStatus::Pending));

    let fetched = repo.get(job.id).await.expect("get");
    assert_eq!(fetched.id, job.id);
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn pg_progress_repository_upsert_and_get() {
    let url = match std::env::var("TEST_PG_URL") {
        Ok(u) => u,
        Err(_) => return,
    };
    let pool = infrastructure::postgres::pool::open_pg_pool(&url).await.unwrap();
    infrastructure::postgres::migrations::run_migrations(&pool).await.unwrap();

    use infrastructure::postgres::{
        progress::PgProgressRepository, resource::PgResourceRepository,
    };
    use domain::{NewResource, ProgressRepository, ResourceRepository, ResourceType};

    let res_repo = PgResourceRepository::new(pool.clone());
    let r = res_repo.create(NewResource {
        title: "prog test resource".to_string(),
        notes: None,
        resource_type: ResourceType::Ebook,
    }).await.expect("create resource");

    let prog_repo = PgProgressRepository::new(pool);
    let result = prog_repo.upsert(&r.id.to_string(), 0.5, Some("halfway"))
        .await.expect("upsert");
    assert_eq!(result.progress, 0.5);
    assert_eq!(result.notes, Some("halfway".to_string()));

    let fetched = prog_repo.get(&r.id.to_string()).await.expect("get").expect("should exist");
    assert_eq!(fetched.progress, 0.5);

    res_repo.delete(r.id).await.ok();
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn pg_tag_repository_create_and_attach() {
    let url = match std::env::var("TEST_PG_URL") {
        Ok(u) => u,
        Err(_) => return,
    };
    let pool = infrastructure::postgres::pool::open_pg_pool(&url).await.unwrap();
    infrastructure::postgres::migrations::run_migrations(&pool).await.unwrap();

    use infrastructure::postgres::{
        resource::PgResourceRepository, tag::PgTagRepository,
    };
    use domain::{NewResource, ResourceRepository, ResourceType};
    use domain::tag::{TagRepository, ResourceTagRepository};

    let res_repo = PgResourceRepository::new(pool.clone());
    let r = res_repo.create(NewResource {
        title: "tag test resource".to_string(),
        notes: None,
        resource_type: ResourceType::Ebook,
    }).await.expect("create resource");

    let tag_repo = PgTagRepository::new(pool);
    let tag = tag_repo.create("pg-test-tag").await.expect("create tag");
    assert_eq!(tag.name, "pg-test-tag");

    tag_repo.attach(&r.id.to_string(), &tag.id).await.expect("attach");
    let tags = tag_repo.tags_for_resource(&r.id.to_string()).await.expect("tags_for_resource");
    assert!(tags.iter().any(|t| t.id == tag.id));

    tag_repo.detach(&r.id.to_string(), &tag.id).await.expect("detach");
    tag_repo.delete(&tag.id).await.expect("delete tag");
    res_repo.delete(r.id).await.ok();
}
