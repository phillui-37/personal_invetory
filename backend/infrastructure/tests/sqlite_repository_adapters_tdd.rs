use domain::sync::SyncJob;
use domain::{
    DomainError, NewEbookMeta, NewResource, NewResourceLocation, NewWebReaderMeta, ResourceType,
    StorageType, UpdateResource,
};
use domain::{NewGameMeta, NewImageMeta, NewVideoMeta};
use futures::executor::block_on;
use infrastructure::{AdapterFactory, DatabaseAdapter};

fn sqlite_bundle() -> infrastructure::AdapterBundle {
    futures::executor::block_on(AdapterFactory::from_url("sqlite://:memory:"))
        .expect("sqlite adapter bundle")
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
        bundle
            .vault_backend
            .save_config(&vault_config)
            .await
            .expect("save config");

        let loaded = bundle
            .vault_backend
            .get_config()
            .await
            .expect("get config")
            .expect("some");
        assert_eq!(loaded.salt, vec![1, 2, 3, 4]);
        assert_eq!(loaded.key_check, vec![5, 6, 7, 8]);
        assert_eq!(loaded.key_check_nonce, vec![9, 10, 11]);
    });
}

#[test]
fn sqlite_vault_backend_blob_store_retrieve_delete() {
    let bundle = sqlite_bundle();
    block_on(async {
        bundle
            .vault_backend
            .store_blob("steam", "api_key", b"encrypted-data", b"nonce-12bytes")
            .await
            .expect("store");

        let (blob, nonce) = bundle
            .vault_backend
            .retrieve_blob("steam", "api_key")
            .await
            .expect("retrieve");
        assert_eq!(blob, b"encrypted-data");
        assert_eq!(nonce, b"nonce-12bytes");

        let platforms = bundle.vault_backend.list_platforms().await.expect("list");
        assert_eq!(platforms, vec!["steam".to_string()]);

        bundle
            .vault_backend
            .delete_credential("steam", "api_key")
            .await
            .expect("delete");
        let result = bundle.vault_backend.retrieve_blob("steam", "api_key").await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_vault_backend_upsert_overwrites() {
    let bundle = sqlite_bundle();
    block_on(async {
        bundle
            .vault_backend
            .store_blob("dlsite", "cookie_jar", b"old", b"nonce1-12byte")
            .await
            .expect("store 1");
        bundle
            .vault_backend
            .store_blob("dlsite", "cookie_jar", b"new", b"nonce2-12byte")
            .await
            .expect("store 2");

        let (blob, nonce) = bundle
            .vault_backend
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

        let job = bundle
            .sync_job_repo
            .create(NewSyncJob {
                platform: "steam".to_string(),
            })
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

        let list = bundle
            .sync_job_repo
            .list_by_platform("steam")
            .await
            .expect("list");
        assert_eq!(list.len(), 1);
    });
}

#[test]
fn sqlite_dedup_warning_create_dismiss_and_exists() {
    let bundle = sqlite_bundle();
    block_on(async {
        use domain::dedup::NewDedupWarning;

        let res_a = bundle
            .resource_repo
            .create(NewResource {
                title: "Zelda TOTK".to_string(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .expect("create a");
        let res_b = bundle
            .resource_repo
            .create(NewResource {
                title: "Zelda Tears".to_string(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .expect("create b");

        let warning = bundle
            .dedup_warning_repo
            .create(NewDedupWarning {
                resource_id_a: res_a.id,
                resource_id_b: res_b.id,
                similarity_score: 0.91,
            })
            .await
            .expect("create warning");

        let pending = bundle
            .dedup_warning_repo
            .list_pending()
            .await
            .expect("list");
        assert_eq!(pending.len(), 1);
        assert!((pending[0].similarity_score - 0.91).abs() < f64::EPSILON);

        let exists = bundle
            .dedup_warning_repo
            .exists_pair(res_a.id, res_b.id)
            .await
            .expect("exists");
        assert!(exists);
        let exists_reverse = bundle
            .dedup_warning_repo
            .exists_pair(res_b.id, res_a.id)
            .await
            .expect("exists reverse");
        assert!(exists_reverse);

        bundle
            .dedup_warning_repo
            .dismiss(warning.id)
            .await
            .expect("dismiss");
        let pending_after = bundle
            .dedup_warning_repo
            .list_pending()
            .await
            .expect("list after");
        assert_eq!(pending_after.len(), 0);
    });
}

#[test]
fn sqlite_device_repository_register_list_and_delink() {
    let bundle = sqlite_bundle();

    block_on(async {
        // Register two devices
        let d1 = bundle
            .device_repo
            .register("desktop-home", Some("Home Desktop"))
            .await
            .expect("register desktop-home");
        assert_eq!(d1.device_id, "desktop-home");
        assert_eq!(d1.device_name.as_deref(), Some("Home Desktop"));
        assert!(d1.delinked_at.is_none());
        assert_eq!(d1.location_count, 0);

        let d2 = bundle
            .device_repo
            .register("laptop-work", None)
            .await
            .expect("register laptop-work");
        assert_eq!(d2.device_id, "laptop-work");
        assert_eq!(d2.device_name, None);

        // List returns both
        let all = bundle
            .device_repo
            .all_with_counts()
            .await
            .expect("all_with_counts");
        assert_eq!(all.len(), 2);

        // Re-register same device_id delinks old, inserts new
        let d1b = bundle
            .device_repo
            .register("desktop-home", Some("Home Desktop v2"))
            .await
            .expect("re-register desktop-home");
        assert_eq!(d1b.device_name.as_deref(), Some("Home Desktop v2"));
        let all2 = bundle
            .device_repo
            .all_with_counts()
            .await
            .expect("all_with_counts after re-register");
        assert_eq!(all2.len(), 2, "re-register must not add a third row");

        // active_by_device_id returns the live binding
        let active = bundle
            .device_repo
            .active_by_device_id("desktop-home")
            .await
            .expect("active_by_device_id");
        assert!(active.is_some());
        assert_eq!(
            active.unwrap().device_name.as_deref(),
            Some("Home Desktop v2")
        );

        // Delink laptop-work
        bundle
            .device_repo
            .delink("laptop-work", chrono::Utc::now())
            .await
            .expect("delink laptop-work");

        // Double-delink returns Conflict
        let err = bundle
            .device_repo
            .delink("laptop-work", chrono::Utc::now())
            .await
            .expect_err("second delink should fail");
        assert!(matches!(err, domain::DomainError::Conflict(_)));

        // Unknown device returns NotFound
        let err2 = bundle
            .device_repo
            .delink("nonexistent", chrono::Utc::now())
            .await
            .expect_err("delink unknown should fail");
        assert!(matches!(err2, domain::DomainError::NotFound(_)));
    });
}

#[test]
fn sqlite_device_repository_location_count_reflects_resource_locations() {
    let bundle = sqlite_bundle();

    block_on(async {
        // Register a device and add resource locations for it
        bundle
            .device_repo
            .register("device-loc", None)
            .await
            .expect("register device-loc");

        // Create a resource and add two locations for device-loc
        let res = bundle
            .resource_repo
            .create(NewResource {
                title: "Loc Test Book".to_string(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create resource");

        bundle
            .location_repo
            .add(
                res.id,
                NewResourceLocation {
                    device_id: "device-loc".to_string(),
                    path_or_url: "/path/a".to_string(),
                    storage_type: StorageType::LocalFs,
                },
            )
            .await
            .expect("add location 1");
        bundle
            .location_repo
            .add(
                res.id,
                NewResourceLocation {
                    device_id: "device-loc".to_string(),
                    path_or_url: "/path/b".to_string(),
                    storage_type: StorageType::LocalFs,
                },
            )
            .await
            .expect("add location 2");

        let all = bundle
            .device_repo
            .all_with_counts()
            .await
            .expect("all_with_counts");
        let device = all
            .iter()
            .find(|d| d.device_id == "device-loc")
            .expect("device-loc should be in list");
        assert_eq!(
            device.location_count, 2,
            "location_count must reflect resource_locations rows"
        );
    });
}

#[test]
fn sqlite_device_repository_register_is_atomic() {
    // Verifies the happy-path atomicity: re-registering succeeds and list stays consistent.
    // (A failed INSERT after UPDATE would leave the device with no active binding.)
    let bundle = sqlite_bundle();

    block_on(async {
        bundle
            .device_repo
            .register("atomic-device", Some("v1"))
            .await
            .expect("first register");

        // Re-register: delinks old, inserts new atomically
        let new_binding = bundle
            .device_repo
            .register("atomic-device", Some("v2"))
            .await
            .expect("re-register");
        assert_eq!(new_binding.device_name.as_deref(), Some("v2"));

        // Active binding must be the new one
        let active = bundle
            .device_repo
            .active_by_device_id("atomic-device")
            .await
            .expect("active_by_device_id");
        assert!(active.is_some(), "device must have an active binding after re-register");
        assert_eq!(active.unwrap().device_name.as_deref(), Some("v2"));

        // all_with_counts still returns exactly one row for this device_id
        let all = bundle
            .device_repo
            .all_with_counts()
            .await
            .expect("all_with_counts");
        assert_eq!(
            all.iter().filter(|d| d.device_id == "atomic-device").count(),
            1,
            "all_with_counts must deduplicate to one row per device_id"
        );
    });
}

#[test]
fn sqlite_progress_repository_upsert_rejects_missing_resource() {
    let bundle = sqlite_bundle();
    let result = block_on(bundle.progress_repo.upsert("missing-resource", 0.25, None));
    assert!(
        matches!(result, Err(DomainError::NotFound(_))),
        "expected NotFound for missing FK, got: {result:?}"
    );
}

#[test]
fn sqlite_progress_repository_upsert_rejects_invalid_progress() {
    let bundle = sqlite_bundle();

    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Check Constraint Test Book".to_string(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create resource");

        let resource_id = resource.id.to_string();
        let result = bundle.progress_repo.upsert(&resource_id, 1.5, None).await;
        assert!(
            matches!(result, Err(DomainError::Conflict(_))),
            "expected Conflict for CHECK constraint violation, got: {result:?}"
        );
    });
}

#[test]
fn sqlite_progress_repository_get_returns_none_when_no_row() {
    let bundle = sqlite_bundle();
    let result = block_on(bundle.progress_repo.get("nonexistent-resource-id")).unwrap();
    assert!(result.is_none());
}

#[test]
fn sqlite_progress_repository_upsert_creates_and_updates() {
    let bundle = sqlite_bundle();

    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Progress Test Book".to_string(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create resource");

        let resource_id = resource.id.to_string();

        // First upsert (create)
        let p1 = bundle
            .progress_repo
            .upsert(&resource_id, 0.25, Some("just started"))
            .await
            .expect("first upsert");
        assert_eq!(p1.resource_id, resource_id);
        assert!((p1.progress - 0.25).abs() < f64::EPSILON);
        assert_eq!(p1.notes, Some("just started".to_string()));

        // Get should return the row
        let fetched = bundle
            .progress_repo
            .get(&resource_id)
            .await
            .expect("get after first upsert")
            .expect("row should exist");
        assert_eq!(fetched.resource_id, resource_id);
        assert!((fetched.progress - 0.25).abs() < f64::EPSILON);

        // Second upsert (update)
        let p2 = bundle
            .progress_repo
            .upsert(&resource_id, 0.75, None)
            .await
            .expect("second upsert");
        assert!((p2.progress - 0.75).abs() < f64::EPSILON);
        assert_eq!(p2.notes, None);

        // Confirm update persisted
        let final_p = bundle
            .progress_repo
            .get(&resource_id)
            .await
            .expect("get after second upsert")
            .expect("row should still exist");
        assert!((final_p.progress - 0.75).abs() < f64::EPSILON);
        assert_eq!(final_p.notes, None);
    });
}

#[test]
fn sqlite_tag_repository_create_and_list() {
    use domain::tag::{TagRepository, ResourceTagRepository};

    let bundle = sqlite_bundle();

    block_on(async {
        // Empty list initially
        let empty = bundle.tag_repo.list().await.expect("list tags on empty db");
        assert!(empty.is_empty());

        // Create two tags
        let sci_fi = bundle.tag_repo.create("sci-fi").await.expect("create sci-fi");
        let backlog = bundle.tag_repo.create("backlog").await.expect("create backlog");

        // list() returns both ordered by name
        let all = bundle.tag_repo.list().await.expect("list after create");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "backlog");
        assert_eq!(all[1].name, "sci-fi");

        // get_by_name
        let found = bundle.tag_repo.get_by_name("sci-fi").await.expect("get_by_name sci-fi");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, sci_fi.id);

        // get_by_id
        let by_id = bundle.tag_repo.get_by_id(&backlog.id).await.expect("get_by_id backlog");
        assert!(by_id.is_some());
        assert_eq!(by_id.unwrap().name, "backlog");

        // get_by_id missing
        let missing = bundle.tag_repo.get_by_id("nonexistent").await.expect("get_by_id missing");
        assert!(missing.is_none());

        // create duplicate → Conflict
        let dup = bundle.tag_repo.create("sci-fi").await;
        assert!(matches!(dup, Err(DomainError::Conflict(_))), "duplicate name must be Conflict, got: {dup:?}");
    });
}

#[test]
fn sqlite_tag_repository_delete_cascades_resource_tags() {
    use domain::tag::{TagRepository, ResourceTagRepository};

    let bundle = sqlite_bundle();

    block_on(async {
        let resource = bundle
            .resource_repo
            .create(NewResource {
                title: "Tag Cascade Test Book".to_string(),
                notes: None,
                resource_type: ResourceType::Ebook,
            })
            .await
            .expect("create resource");
        let resource_id = resource.id.to_string();

        let tag = bundle.tag_repo.create("to-delete").await.expect("create tag");

        bundle.resource_tag_repo.attach(&resource_id, &tag.id).await.expect("attach tag");

        // Confirm attached
        let tags_before = bundle.resource_tag_repo.tags_for_resource(&resource_id).await.expect("tags before delete");
        assert_eq!(tags_before.len(), 1);

        // Delete tag cascades resource_tags
        bundle.tag_repo.delete(&tag.id).await.expect("delete tag");

        let tags_after = bundle.resource_tag_repo.tags_for_resource(&resource_id).await.expect("tags after delete");
        assert!(tags_after.is_empty(), "resource_tags must be empty after tag deletion");

        let gone = bundle.tag_repo.get_by_id(&tag.id).await.expect("get_by_id after delete");
        assert!(gone.is_none(), "tag must not be found after deletion");

        // delete unknown → NotFound
        let err = bundle.tag_repo.delete("nonexistent-tag").await;
        assert!(matches!(err, Err(DomainError::NotFound(_))), "deleting unknown tag must be NotFound");
    });
}

#[test]
fn sqlite_resource_tag_repository_attach_detach_and_filter() {
    use domain::tag::{TagRepository, ResourceTagRepository};

    let bundle = sqlite_bundle();

    block_on(async {
        // Create 2 resources and 2 tags
        let r1 = bundle.resource_repo.create(NewResource {
            title: "Resource One".to_string(),
            notes: None,
            resource_type: ResourceType::Ebook,
        }).await.expect("create r1");
        let r2 = bundle.resource_repo.create(NewResource {
            title: "Resource Two".to_string(),
            notes: None,
            resource_type: ResourceType::Ebook,
        }).await.expect("create r2");

        let tag_a = bundle.tag_repo.create("tag-a").await.expect("create tag-a");
        let tag_b = bundle.tag_repo.create("tag-b").await.expect("create tag-b");

        let r1_id = r1.id.to_string();
        let r2_id = r2.id.to_string();

        // Attach tag A to both resources, tag B to r1 only
        bundle.resource_tag_repo.attach(&r1_id, &tag_a.id).await.expect("attach tag-a to r1");
        bundle.resource_tag_repo.attach(&r2_id, &tag_a.id).await.expect("attach tag-a to r2");
        bundle.resource_tag_repo.attach(&r1_id, &tag_b.id).await.expect("attach tag-b to r1");

        // tags_for_resource(r1) = [tag_a, tag_b] ordered by name
        let r1_tags = bundle.resource_tag_repo.tags_for_resource(&r1_id).await.expect("tags for r1");
        assert_eq!(r1_tags.len(), 2);
        assert_eq!(r1_tags[0].name, "tag-a");
        assert_eq!(r1_tags[1].name, "tag-b");

        // resource_ids_with_tag_id(tag_a) returns both resource IDs
        let with_tag_a = bundle.resource_tag_repo.resource_ids_with_tag_id(&tag_a.id).await.expect("resource_ids_with_tag_id");
        assert_eq!(with_tag_a.len(), 2);
        assert!(with_tag_a.contains(&r1_id));
        assert!(with_tag_a.contains(&r2_id));

        // Detach tag B from r1
        bundle.resource_tag_repo.detach(&r1_id, &tag_b.id).await.expect("detach tag-b from r1");

        // r1 now only has tag_a
        let r1_tags_after = bundle.resource_tag_repo.tags_for_resource(&r1_id).await.expect("tags for r1 after detach");
        assert_eq!(r1_tags_after.len(), 1);
        assert_eq!(r1_tags_after[0].name, "tag-a");

        // detach same association again → NotFound
        let err = bundle.resource_tag_repo.detach(&r1_id, &tag_b.id).await;
        assert!(matches!(err, Err(DomainError::NotFound(_))), "detach non-existent must be NotFound, got: {err:?}");

        // re-attach existing association is idempotent
        bundle.resource_tag_repo.attach(&r1_id, &tag_a.id).await.expect("re-attach tag-a to r1 must be idempotent");

        // Verify no duplicate association was created
        let r1_tags_after_reattach = bundle.resource_tag_repo.tags_for_resource(&r1_id).await.expect("tags for r1 after reattach");
        assert_eq!(r1_tags_after_reattach.len(), 1, "re-attach should not create duplicate");
        assert_eq!(r1_tags_after_reattach[0].name, "tag-a");

        // Stronger check: verify resource_ids_with_tag_id still has exactly 2 unique resource IDs for tag_a
        let with_tag_a_after = bundle.resource_tag_repo.resource_ids_with_tag_id(&tag_a.id).await.expect("resource_ids_with_tag_id after reattach");
        assert_eq!(with_tag_a_after.len(), 2, "tag-a should still be attached to exactly 2 resources");
        assert!(with_tag_a_after.contains(&r1_id));
        assert!(with_tag_a_after.contains(&r2_id));
    });
}

// P7-A compile test: verify the postgres module tree (pool, migrations) is accessible
#[test]
fn pg_module_tree_compiles() {
    // Assign function pointers to verify the modules compile and are reachable
    let _pool_fn = infrastructure::postgres::pool::open_pg_pool;
    let _mig_fn = infrastructure::postgres::migrations::run_migrations;
    let _ = (_pool_fn, _mig_fn);
}
