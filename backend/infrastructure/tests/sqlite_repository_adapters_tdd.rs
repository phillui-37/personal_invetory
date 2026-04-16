use domain::{
    DomainError, NewEbookMeta, NewResource, NewResourceLocation, NewWebReaderMeta, ResourceType,
    StorageType, UpdateResource,
};
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
