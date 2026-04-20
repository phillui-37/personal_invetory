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
}
