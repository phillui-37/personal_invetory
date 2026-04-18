use domain::{
    DomainError, NewEbookMeta, NewResource, NewResourceLocation, NewWebReaderMeta, ResourceType,
    StorageType, UpdateResource,
};
use domain::{NewImageMeta, NewVideoMeta, NewGameMeta};
use domain::sync::SyncJob;
use futures::executor::block_on;
use infrastructure::{AdapterFactory, DatabaseAdapter};

fn sqlite_bundle() -> infrastructure::AdapterBundle {
    AdapterFactory::from_url("sqlite://:memory:").expect("sqlite adapter bundle")
}

#[test]
fn sqlite_resource_repository_crud_search_and_ordering() {
    let bundle = sqlite_bundle();
    assert!(matches!(bundle.database, DatabaseAdapter::Sqlite));

    block_on(async {
        let alpha = bundle
            .resource_repo
            .create(NewResource {
                title: "Alpha Rust".to_string(),
                notes: Some("note-a".to_string()),
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create alpha");
        let beta = bundle
            .resource_repo
            .create(NewResource {
                title: "beta rust".to_string(),
                notes: None,
                resource_type: ResourceType::WebReader,
            })
            .await
            .expect("create beta");

        let list = bundle.resource_repo.list().await.expect("list resources");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].title, "Alpha Rust");
        assert_eq!(list[1].title, "beta rust");

        let searched = bundle
            .resource_repo
            .search("RUST")
            .await
            .expect("case-insensitive search");
        assert_eq!(searched.len(), 2);
        assert_eq!(searched[0].id, alpha.id);
        assert_eq!(searched[1].id, beta.id);

        let empty = bundle.resource_repo.search("  ").await;
        assert!(matches!(empty, Err(DomainError::ValidationError(_))));

        let duplicate = bundle
            .resource_repo
            .create(NewResource {
                title: "alpha rust".to_string(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await;
        assert!(matches!(duplicate, Err(DomainError::Conflict(_))));

        let updated = bundle
            .resource_repo
            .update(
                alpha.id,
                UpdateResource {
                    title: Some("Alpha Rust 2".to_string()),
                    notes: Some("updated".to_string()),
                },
            )
            .await
            .expect("update resource");
        assert_eq!(updated.title, "Alpha Rust 2");
        assert_eq!(updated.notes.as_deref(), Some("updated"));

        bundle
            .resource_repo
            .delete(beta.id)
            .await
            .expect("delete beta");
        let deleted = bundle.resource_repo.get_by_id(beta.id).await;
        assert!(matches!(deleted, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_meta_and_location_repositories_map_domain_errors() {
    let bundle = sqlite_bundle();

    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Book One".to_string(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create resource");

        let no_meta_resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Book Two".to_string(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create second resource");

        let ebook_meta = bundle
            .ebook_meta_repo
            .upsert(
                resource.id,
                NewEbookMeta {
                    author: Some("Someone".to_string()),
                    isbn: Some("isbn".to_string()),
                    publisher: None,
                    language: Some("en".to_string()),
                    file_format: Some("epub".to_string()),
                },
            )
            .await
            .expect("upsert ebook meta");
        assert_eq!(ebook_meta.author.as_deref(), Some("Someone"));

        let fetched_ebook_meta = bundle
            .ebook_meta_repo
            .get(resource.id)
            .await
            .expect("get ebook meta");
        assert_eq!(fetched_ebook_meta.isbn.as_deref(), Some("isbn"));

        let web_meta = bundle
            .web_reader_meta_repo
            .upsert(
                resource.id,
                NewWebReaderMeta {
                    url: "https://example.com/read".to_string(),
                    site_name: Some("Example".to_string()),
                    last_checked_chapter: Some("ch-1".to_string()),
                    check_interval_secs: None,
                    last_checked_at: None,
                    progress_css_selector: None,
                },
            )
            .await
            .expect("upsert web meta");
        assert_eq!(web_meta.site_name.as_deref(), Some("Example"));

        let missing_meta = bundle.ebook_meta_repo.get(no_meta_resource.id).await;
        assert!(matches!(missing_meta, Err(DomainError::NotFound(_))));

        let added_location = bundle
            .location_repo
            .add(
                resource.id,
                NewResourceLocation {
                    device_id: "device-1".to_string(),
                    path_or_url: "/books/book-one.epub".to_string(),
                    storage_type: StorageType::LocalFs,
                },
            )
            .await
            .expect("add location");

        let listed_locations = bundle
            .location_repo
            .list(resource.id)
            .await
            .expect("list locations");
        assert_eq!(listed_locations.len(), 1);
        assert_eq!(listed_locations[0].id, added_location.id);

        bundle
            .location_repo
            .remove(resource.id, added_location.id)
            .await
            .expect("remove location");

        let missing_remove = bundle
            .location_repo
            .remove(resource.id, added_location.id)
            .await;
        assert!(matches!(missing_remove, Err(DomainError::NotFound(_))));

        bundle
            .resource_repo
            .delete(no_meta_resource.id)
            .await
            .expect("delete second resource");

        let fk_miss = bundle
            .location_repo
            .add(
                no_meta_resource.id,
                NewResourceLocation {
                    device_id: "device-404".to_string(),
                    path_or_url: "/missing".to_string(),
                    storage_type: StorageType::LocalFs,
                },
            )
            .await;
        assert!(matches!(fk_miss, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_chapter_check_repository_create_and_list() {
    let bundle = sqlite_bundle();

    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Web Comic".to_string(),
                notes: None,
                resource_type: ResourceType::WebReader,
            })
            .await
            .expect("create resource");

        let check = bundle
            .chapter_check_repo
            .create(resource.id, true, Some("ch-42".to_string()), None)
            .await
            .expect("create chapter check");
        assert_eq!(check.resource_id, resource.id);
        assert!(check.has_new_chapter);
        assert_eq!(check.latest_chapter.as_deref(), Some("ch-42"));
        assert!(check.error_message.is_none());

        let err_check = bundle
            .chapter_check_repo
            .create(resource.id, false, None, Some("timeout".to_string()))
            .await
            .expect("create error check");
        assert!(!err_check.has_new_chapter);
        assert_eq!(err_check.error_message.as_deref(), Some("timeout"));

        let listed = bundle
            .chapter_check_repo
            .list(resource.id)
            .await
            .expect("list checks");
        assert_eq!(listed.len(), 2);
        // ordered by checked_at DESC, so err_check was inserted last
        assert_eq!(listed[0].id, err_check.id);
        assert_eq!(listed[1].id, check.id);
    });
}

#[test]
fn sqlite_image_meta_repository_upsert_and_get() {
    let bundle = sqlite_bundle();
    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Test Image".to_string(),
                notes: None,
                resource_type: ResourceType::Image,
            })
            .await
            .expect("create resource");

        let meta = bundle
            .image_meta_repo
            .upsert(
                resource.id,
                NewImageMeta {
                    width: Some(1920),
                    height: Some(1080),
                    file_format: Some("png".to_string()),
                    file_size_bytes: Some(204800),
                },
            )
            .await
            .expect("upsert image meta");
        assert_eq!(meta.width, Some(1920));

        let fetched = bundle
            .image_meta_repo
            .get(resource.id)
            .await
            .expect("get image meta");
        assert_eq!(fetched.file_format.as_deref(), Some("png"));
        assert_eq!(fetched.file_size_bytes, Some(204800));
    });
}

#[test]
fn sqlite_video_meta_repository_upsert_and_get() {
    let bundle = sqlite_bundle();
    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Test Video".to_string(),
                notes: None,
                resource_type: ResourceType::Video,
            })
            .await
            .expect("create resource");

        let meta = bundle
            .video_meta_repo
            .upsert(
                resource.id,
                NewVideoMeta {
                    duration_secs: Some(3600),
                    file_format: Some("mkv".to_string()),
                    resolution: Some("1920x1080".to_string()),
                    file_size_bytes: Some(1_000_000),
                },
            )
            .await
            .expect("upsert video meta");
        assert_eq!(meta.duration_secs, Some(3600));

        let fetched = bundle
            .video_meta_repo
            .get(resource.id)
            .await
            .expect("get video meta");
        assert_eq!(fetched.resolution.as_deref(), Some("1920x1080"));
    });
}

#[test]
fn sqlite_game_meta_repository_upsert_and_get() {
    let bundle = sqlite_bundle();
    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Test Game".to_string(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .expect("create resource");

        let meta = bundle
            .game_meta_repo
            .upsert(
                resource.id,
                NewGameMeta {
                    platform: Some("Nintendo Switch".to_string()),
                    store: Some("eShop".to_string()),
                    developer: None,
                    publisher: None,
                    manual_notes: Some("physical cartridge".to_string()),
                },
            )
            .await
            .expect("upsert game meta");
        assert_eq!(meta.platform.as_deref(), Some("Nintendo Switch"));

        let fetched = bundle
            .game_meta_repo
            .get(resource.id)
            .await
            .expect("get game meta");
        assert_eq!(fetched.manual_notes.as_deref(), Some("physical cartridge"));
    });
}

#[test]
fn sqlite_missing_image_meta_returns_not_found() {
    let bundle = sqlite_bundle();
    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "No Meta Image".to_string(),
                notes: None,
                resource_type: ResourceType::Image,
            })
            .await
            .expect("create resource");
        let result = bundle.image_meta_repo.get(resource.id).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_missing_video_meta_returns_not_found() {
    let bundle = sqlite_bundle();
    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "No Meta Video".to_string(),
                notes: None,
                resource_type: ResourceType::Video,
            })
            .await
            .expect("create resource");
        let result = bundle.video_meta_repo.get(resource.id).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_missing_game_meta_returns_not_found() {
    let bundle = sqlite_bundle();
    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "No Meta Game".to_string(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .expect("create resource");
        let result = bundle.game_meta_repo.get(resource.id).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_notification_repository_create_list_and_mark_read() {
    let bundle = sqlite_bundle();

    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Manga Site".to_string(),
                notes: None,
                resource_type: ResourceType::WebReader,
            })
            .await
            .expect("create resource");

        let n1 = bundle
            .notification_repo
            .create(resource.id, "New chapter 10!".to_string())
            .await
            .expect("create notification 1");
        assert!(!n1.read);

        let n2 = bundle
            .notification_repo
            .create(resource.id, "New chapter 11!".to_string())
            .await
            .expect("create notification 2");

        // list all
        let all = bundle
            .notification_repo
            .list(false)
            .await
            .expect("list all notifications");
        assert_eq!(all.len(), 2);

        // list unread only
        let unread = bundle
            .notification_repo
            .list(true)
            .await
            .expect("list unread");
        assert_eq!(unread.len(), 2);

        // mark one read
        bundle
            .notification_repo
            .mark_read(n1.id)
            .await
            .expect("mark n1 read");

        let unread_after = bundle
            .notification_repo
            .list(true)
            .await
            .expect("list unread after mark");
        assert_eq!(unread_after.len(), 1);
        assert_eq!(unread_after[0].id, n2.id);

        // mark_read on missing id returns NotFound
        let missing = bundle
            .notification_repo
            .mark_read(uuid::Uuid::new_v4())
            .await;
        assert!(matches!(missing, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_vault_backend_config_roundtrip() {
    let bundle = sqlite_bundle();
    block_on(async {
        let config = bundle.vault_backend.get_config().await.expect("get config");
        assert!(config.is_none(), "no config initially");

        let vault_config = domain::vault::VaultConfig {
            salt: vec![1, 2, 3, 4],
            key_check: vec![5, 6, 7, 8],
            key_check_nonce: vec![9, 10, 11],
        };
        bundle.vault_backend.save_config(&vault_config).await.expect("save config");

        let loaded = bundle.vault_backend.get_config().await.expect("get config").expect("some");
        assert_eq!(loaded.salt, vec![1, 2, 3, 4]);
        assert_eq!(loaded.key_check, vec![5, 6, 7, 8]);
        assert_eq!(loaded.key_check_nonce, vec![9, 10, 11]);
    });
}

#[test]
fn sqlite_vault_backend_blob_store_retrieve_delete() {
    let bundle = sqlite_bundle();
    block_on(async {
        bundle.vault_backend
            .store_blob("steam", "api_key", b"encrypted-data", b"nonce-12bytes")
            .await
            .expect("store");

        let (blob, nonce) = bundle.vault_backend
            .retrieve_blob("steam", "api_key")
            .await
            .expect("retrieve");
        assert_eq!(blob, b"encrypted-data");
        assert_eq!(nonce, b"nonce-12bytes");

        let platforms = bundle.vault_backend.list_platforms().await.expect("list");
        assert_eq!(platforms, vec!["steam".to_string()]);

        bundle.vault_backend.delete_credential("steam", "api_key").await.expect("delete");
        let result = bundle.vault_backend.retrieve_blob("steam", "api_key").await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_vault_backend_upsert_overwrites() {
    let bundle = sqlite_bundle();
    block_on(async {
        bundle.vault_backend
            .store_blob("dlsite", "cookie_jar", b"old", b"nonce1-12byte")
            .await
            .expect("store 1");
        bundle.vault_backend
            .store_blob("dlsite", "cookie_jar", b"new", b"nonce2-12byte")
            .await
            .expect("store 2");

        let (blob, nonce) = bundle.vault_backend
            .retrieve_blob("dlsite", "cookie_jar")
            .await
            .expect("retrieve");
        assert_eq!(blob, b"new");
        assert_eq!(nonce, b"nonce2-12byte");
    });
}

#[test]
fn sqlite_sync_job_create_update_and_list() {
    let bundle = sqlite_bundle();
    block_on(async {
        use domain::sync::{NewSyncJob, SyncJobStatus};

        let job = bundle.sync_job_repo
            .create(NewSyncJob { platform: "steam".to_string() })
            .await
            .expect("create");
        assert_eq!(job.platform, "steam");
        assert_eq!(job.status, SyncJobStatus::Pending);
        assert_eq!(job.items_found, 0);

        let mut updated = SyncJob {
            id: job.id,
            platform: job.platform.clone(),
            status: SyncJobStatus::Completed,
            started_at: job.started_at,
            completed_at: job.completed_at,
            items_found: 10,
            items_created: 8,
            items_skipped: 2,
            items_failed: 0,
            error_message: None,
            created_at: job.created_at,
        };
        bundle.sync_job_repo.update(&updated).await.expect("update");

        let loaded = bundle.sync_job_repo.get(updated.id).await.expect("get");
        assert_eq!(loaded.status, SyncJobStatus::Completed);
        assert_eq!(loaded.items_found, 10);
        assert_eq!(loaded.items_created, 8);
        assert_eq!(loaded.items_skipped, 2);

        let list = bundle.sync_job_repo.list_by_platform("steam").await.expect("list");
        assert_eq!(list.len(), 1);
    });
}

#[test]
fn sqlite_dedup_warning_create_dismiss_and_exists() {
    let bundle = sqlite_bundle();
    block_on(async {
        use domain::dedup::NewDedupWarning;

        let res_a = bundle.resource_repo
            .create(NewResource { title: "Zelda TOTK".to_string(), notes: None, resource_type: ResourceType::Game })
            .await.expect("create a");
        let res_b = bundle.resource_repo
            .create(NewResource { title: "Zelda Tears".to_string(), notes: None, resource_type: ResourceType::Game })
            .await.expect("create b");

        let warning = bundle.dedup_warning_repo
            .create(NewDedupWarning {
                resource_id_a: res_a.id,
                resource_id_b: res_b.id,
                similarity_score: 0.91,
            })
            .await.expect("create warning");

        let pending = bundle.dedup_warning_repo.list_pending().await.expect("list");
        assert_eq!(pending.len(), 1);
        assert!((pending[0].similarity_score - 0.91).abs() < f64::EPSILON);

        let exists = bundle.dedup_warning_repo.exists_pair(res_a.id, res_b.id).await.expect("exists");
        assert!(exists);
        let exists_reverse = bundle.dedup_warning_repo.exists_pair(res_b.id, res_a.id).await.expect("exists reverse");
        assert!(exists_reverse);

        bundle.dedup_warning_repo.dismiss(warning.id).await.expect("dismiss");
        let pending_after = bundle.dedup_warning_repo.list_pending().await.expect("list after");
        assert_eq!(pending_after.len(), 0);
    });
}
