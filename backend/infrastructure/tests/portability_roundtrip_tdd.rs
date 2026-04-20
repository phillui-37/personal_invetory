use infrastructure::{
    export_canonical_snapshot_sqlite, import_canonical_snapshot_sqlite, normalize_ebook_meta_rows,
    normalize_resource_location_rows, normalize_resource_rows, normalize_web_reader_meta_rows,
    postgres_parity_hooks, CanonicalResourceSnapshot, EbookMetaRow, ResourceLocationRow,
    ResourceRow, WebReaderMetaRow,
};
use rusqlite::Connection;

const SQLITE_MIGRATIONS: [&str; 21] = [
    include_str!("../migrations/0001_create_resources.sql"),
    include_str!("../migrations/0002_create_ebook_metas.sql"),
    include_str!("../migrations/0003_create_web_reader_metas.sql"),
    include_str!("../migrations/0004_create_resource_locations.sql"),
    include_str!("../migrations/0005_create_devices.sql"),
    include_str!("../migrations/0006_alter_web_reader_metas_add_check_fields.sql"),
    include_str!("../migrations/0007_create_chapter_checks.sql"),
    include_str!("../migrations/0008_create_site_configs.sql"),
    include_str!("../migrations/0009_create_notifications.sql"),
    include_str!("../migrations/0010_create_image_metas.sql"),
    include_str!("../migrations/0011_create_video_metas.sql"),
    include_str!("../migrations/0012_create_game_metas.sql"),
    include_str!("../migrations/0013_create_vault_config.sql"),
    include_str!("../migrations/0014_create_credentials.sql"),
    include_str!("../migrations/0015_create_sync_jobs.sql"),
    include_str!("../migrations/0016_create_dedup_warnings.sql"),
    include_str!("../migrations/0017_alter_devices_add_name.sql"),
    include_str!("../migrations/0018_create_device_location_view.sql"),
    include_str!("../migrations/0019_create_resource_progress.sql"),
    include_str!("../migrations/0020_create_tags.sql"),
    include_str!("../migrations/0021_create_resource_tags.sql"),
];

fn apply_sqlite_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    for migration in SQLITE_MIGRATIONS {
        conn.execute_batch(migration)?;
    }
    Ok(())
}

fn seed_roundtrip_fixture(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO resources (id, title, notes, resource_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            "resource-b",
            "B title",
            Some("note-b"),
            "ebook",
            "2026-04-16T09:00:00Z",
            "2026-04-16T09:01:00Z",
        ),
    )?;
    conn.execute(
        "INSERT INTO resources (id, title, notes, resource_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            "resource-a",
            "A title",
            None::<&str>,
            "web_reader",
            "2026-04-16T10:00:00Z",
            "2026-04-16T10:01:00Z",
        ),
    )?;

    conn.execute(
        "INSERT INTO ebook_metas (resource_id, author, isbn, publisher, language, file_format)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            "resource-b",
            Some("Author B"),
            Some("isbn-b"),
            Some("Publisher B"),
            Some("en"),
            Some("epub"),
        ),
    )?;

    conn.execute(
        "INSERT INTO web_reader_metas (resource_id, url, site_name, last_checked_chapter)
         VALUES (?1, ?2, ?3, ?4)",
        (
            "resource-a",
            "https://example.com/chapter-1",
            Some("Example"),
            Some("chapter-42"),
        ),
    )?;

    conn.execute(
        "INSERT INTO resource_locations (id, resource_id, device_id, path_or_url, storage_type)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            "loc-2",
            "resource-b",
            "device-2",
            "/books/b.epub",
            "LocalFs",
        ),
    )?;
    conn.execute(
        "INSERT INTO resource_locations (id, resource_id, device_id, path_or_url, storage_type)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            "loc-1",
            "resource-a",
            "device-1",
            "https://example.com/chapter-1",
            "Platform",
        ),
    )?;

    Ok(())
}

#[test]
fn sqlite_roundtrip_preserves_canonical_snapshot() {
    let source = Connection::open_in_memory().expect("open source sqlite");
    let mut target = Connection::open_in_memory().expect("open target sqlite");

    apply_sqlite_migrations(&source).expect("apply source migrations");
    apply_sqlite_migrations(&target).expect("apply target migrations");
    seed_roundtrip_fixture(&source).expect("seed source fixture");

    let exported = export_canonical_snapshot_sqlite(&source).expect("export source snapshot");
    import_canonical_snapshot_sqlite(&mut target, &exported).expect("import snapshot");

    let re_exported = export_canonical_snapshot_sqlite(&target).expect("re-export target snapshot");
    assert_eq!(exported, re_exported);
}

#[test]
fn normalization_helpers_produce_stable_sorted_snapshot() {
    let unsorted = CanonicalResourceSnapshot {
        resources: vec![
            ResourceRow {
                id: "resource-z".to_string(),
                title: "Z".to_string(),
                notes: None,
                resource_type: "ebook".to_string(),
                created_at: "2026-04-16T10:00:00Z".to_string(),
                updated_at: "2026-04-16T10:00:01Z".to_string(),
            },
            ResourceRow {
                id: "resource-a".to_string(),
                title: "A".to_string(),
                notes: Some("note".to_string()),
                resource_type: "web_reader".to_string(),
                created_at: "2026-04-16T09:00:00Z".to_string(),
                updated_at: "2026-04-16T09:00:01Z".to_string(),
            },
        ],
        ebook_metas: vec![
            EbookMetaRow {
                resource_id: "resource-z".to_string(),
                author: None,
                isbn: Some("b".to_string()),
                publisher: None,
                language: None,
                file_format: Some("epub".to_string()),
            },
            EbookMetaRow {
                resource_id: "resource-a".to_string(),
                author: Some("Author A".to_string()),
                isbn: Some("a".to_string()),
                publisher: None,
                language: None,
                file_format: Some("pdf".to_string()),
            },
        ],
        web_reader_metas: vec![
            WebReaderMetaRow {
                resource_id: "resource-z".to_string(),
                url: "https://example.com/z".to_string(),
                site_name: None,
                last_checked_chapter: Some("z".to_string()),
            },
            WebReaderMetaRow {
                resource_id: "resource-a".to_string(),
                url: "https://example.com/a".to_string(),
                site_name: Some("A".to_string()),
                last_checked_chapter: Some("a".to_string()),
            },
        ],
        resource_locations: vec![
            ResourceLocationRow {
                id: "loc-z".to_string(),
                resource_id: "resource-z".to_string(),
                device_id: "device-z".to_string(),
                path_or_url: "z".to_string(),
                storage_type: "LocalFs".to_string(),
            },
            ResourceLocationRow {
                id: "loc-a".to_string(),
                resource_id: "resource-a".to_string(),
                device_id: "device-a".to_string(),
                path_or_url: "a".to_string(),
                storage_type: "Platform".to_string(),
            },
        ],
        ..Default::default()
    };

    let normalized = CanonicalResourceSnapshot {
        resources: normalize_resource_rows(unsorted.resources),
        ebook_metas: normalize_ebook_meta_rows(unsorted.ebook_metas),
        web_reader_metas: normalize_web_reader_meta_rows(unsorted.web_reader_metas),
        resource_locations: normalize_resource_location_rows(unsorted.resource_locations),
        ..Default::default()
    };

    assert_eq!(normalized.resources[0].id, "resource-a");
    assert_eq!(normalized.ebook_metas[0].resource_id, "resource-a");
    assert_eq!(normalized.web_reader_metas[0].resource_id, "resource-a");
    assert_eq!(normalized.resource_locations[0].id, "loc-a");
}

#[test]
fn postgres_parity_hooks_are_concrete_and_non_todo() {
    let hooks = postgres_parity_hooks();

    assert!(!hooks.export_contract.contains("TODO("));
    assert!(!hooks.import_contract.contains("TODO("));
    assert!(hooks.export_contract.contains("ORDER BY"));
    assert!(hooks.import_contract.contains("ON CONFLICT"));
}
