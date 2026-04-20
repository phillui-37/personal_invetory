mod factory;
mod portability;
pub mod postgres;
pub mod sqlite;
pub mod crypto;

#[cfg(feature = "firebase")]
pub mod fcm;

pub use factory::{resolve_search_strategy, AdapterBundle, AdapterFactory, DatabaseAdapter};
pub use sqlite::progress::SqliteProgressRepository;
pub use sqlite::tag::SqliteTagRepository;
pub use portability::{
    export_canonical_snapshot_sqlite, import_canonical_snapshot_sqlite, normalize_ebook_meta_rows,
    normalize_resource_location_rows, normalize_resource_rows, normalize_web_reader_meta_rows,
    postgres_parity_hooks, CanonicalResourceSnapshot, EbookMetaRow, PortabilityBackend,
    PostgresParityHooks, ResourceLocationRow, ResourceRow, WebReaderMetaRow,
};

use rusqlite::{Connection, OptionalExtension};

pub fn infrastructure_ready() -> bool {
    domain::domain_ready() && plugins::plugins_ready()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceBinding {
    pub id: String,
    pub device_id: String,
    pub owner_id: String,
    pub linked_at: String,
    pub delinked_at: Option<String>,
}

pub fn build_device_binding_id(device_id: &str, owner_id: &str, linked_at: &str) -> String {
    format!("{device_id}:{owner_id}:{linked_at}")
}

pub fn register_device_owner(
    conn: &Connection,
    device_id: &str,
    owner_id: &str,
    linked_at: &str,
) -> rusqlite::Result<DeviceBinding> {
    conn.execute(
        "UPDATE devices
         SET delinked_at = ?1
         WHERE device_id = ?2
           AND delinked_at IS NULL",
        [linked_at, device_id],
    )?;

    let binding_id = build_device_binding_id(device_id, owner_id, linked_at);
    conn.execute(
        "INSERT INTO devices (id, device_id, owner_id, linked_at, delinked_at)
         VALUES (?1, ?2, ?3, ?4, NULL)",
        [&binding_id, device_id, owner_id, linked_at],
    )?;

    Ok(DeviceBinding {
        id: binding_id,
        device_id: device_id.to_string(),
        owner_id: owner_id.to_string(),
        linked_at: linked_at.to_string(),
        delinked_at: None,
    })
}

pub fn active_owner_by_device_id(
    conn: &Connection,
    device_id: &str,
) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT owner_id
         FROM devices
         WHERE device_id = ?1
           AND delinked_at IS NULL
         ORDER BY linked_at DESC
         LIMIT 1",
        [device_id],
        |row| row.get(0),
    )
    .optional()
}

#[cfg(test)]
mod tests {
    use domain::DomainError;
    use rusqlite::Connection;

    #[test]
    fn migrations_create_expected_tables() {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite");
        super::sqlite::apply_sqlite_migrations(&conn).expect("apply migrations");

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .expect("prepare table query");
        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query tables")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect table rows");

        assert!(table_names.contains(&"resources".to_string()));
        assert!(table_names.contains(&"ebook_metas".to_string()));
        assert!(table_names.contains(&"web_reader_metas".to_string()));
        assert!(table_names.contains(&"resource_locations".to_string()));
        assert!(table_names.contains(&"devices".to_string()));
        assert!(table_names.contains(&"image_metas".to_string()));
        assert!(table_names.contains(&"video_metas".to_string()));
        assert!(table_names.contains(&"game_metas".to_string()));
        assert!(table_names.contains(&"vault_config".to_string()));
        assert!(table_names.contains(&"credentials".to_string()));
        assert!(table_names.contains(&"sync_jobs".to_string()));
        assert!(table_names.contains(&"dedup_warnings".to_string()));
        assert!(table_names.contains(&"resource_progress".to_string()));
        assert!(table_names.contains(&"tags".to_string()));
        assert!(table_names.contains(&"resource_tags".to_string()));
    }

    #[tokio::test]
    async fn adapter_factory_recognizes_sqlite_prefix() {
        let bundle = super::AdapterFactory::from_url("sqlite://:memory:")
            .await
            .expect("sqlite adapter bundle");
        assert!(matches!(bundle.database, super::DatabaseAdapter::Sqlite));
    }

    #[tokio::test]
    async fn adapter_factory_rejects_unknown_prefix() {
        let error = match super::AdapterFactory::from_url("mysql://localhost/inventory").await {
            Ok(_) => panic!("unknown prefix should return an error"),
            Err(error) => error,
        };
        assert!(
            matches!(error, DomainError::ValidationError(message) if message.contains("Unsupported database URL prefix"))
        );
    }

    #[test]
    fn resolve_search_strategy_defaults_to_like() {
        let strategy = super::resolve_search_strategy(&super::DatabaseAdapter::Sqlite, None);
        assert_eq!(strategy, domain::SearchStrategyKind::Like);
    }

    #[test]
    fn resolve_search_strategy_accepts_fuzzy_from_config() {
        let strategy =
            super::resolve_search_strategy(&super::DatabaseAdapter::Postgres, Some("fuzzy"));
        assert_eq!(strategy, domain::SearchStrategyKind::Fuzzy);
    }

    #[test]
    fn first_device_registration_sets_active_owner() {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite");
        super::sqlite::apply_sqlite_migrations(&conn).expect("apply migrations");

        super::register_device_owner(&conn, "device-1", "owner-a", "2026-04-16T12:00:00Z")
            .expect("register first owner");

        let active_owner =
            super::active_owner_by_device_id(&conn, "device-1").expect("query active owner");
        assert_eq!(active_owner, Some("owner-a".to_string()));
    }

    #[test]
    fn reregistering_device_delinks_previous_owner_and_activates_new_owner() {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite");
        super::sqlite::apply_sqlite_migrations(&conn).expect("apply migrations");

        super::register_device_owner(&conn, "device-1", "owner-a", "2026-04-16T12:00:00Z")
            .expect("register first owner");
        super::register_device_owner(&conn, "device-1", "owner-b", "2026-04-16T13:00:00Z")
            .expect("register replacement owner");

        let active_owner =
            super::active_owner_by_device_id(&conn, "device-1").expect("query active owner");
        assert_eq!(active_owner, Some("owner-b".to_string()));

        let mut stmt = conn
            .prepare(
                "SELECT owner_id, linked_at, delinked_at
                 FROM devices
                 WHERE device_id = ?1
                 ORDER BY linked_at",
            )
            .expect("prepare device history query");
        let rows: Vec<(String, String, Option<String>)> = stmt
            .query_map(["device-1"], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .expect("query device history")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect device history");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "owner-a");
        assert_eq!(rows[0].2, Some("2026-04-16T13:00:00Z".to_string()));
        assert_eq!(rows[1].0, "owner-b");
        assert_eq!(rows[1].2, None);
    }

    #[test]
    fn repeated_registration_keeps_single_active_owner_for_device_id() {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite");
        super::sqlite::apply_sqlite_migrations(&conn).expect("apply migrations");

        super::register_device_owner(&conn, "device-1", "owner-a", "2026-04-16T12:00:00Z")
            .expect("register first owner");
        super::register_device_owner(&conn, "device-1", "owner-a", "2026-04-16T13:00:00Z")
            .expect("register same owner again");

        let active_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM devices WHERE device_id = ?1 AND delinked_at IS NULL",
                ["device-1"],
                |row| row.get(0),
            )
            .expect("count active device owner rows");
        assert_eq!(active_count, 1);
    }

    #[test]
    fn resources_title_is_case_insensitively_unique() {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite");
        super::sqlite::apply_sqlite_migrations(&conn).expect("apply migrations");

        conn.execute(
            "INSERT INTO resources (id, title, notes, resource_type, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5)",
            (
                "resource-1",
                "My Book",
                "ebook",
                "2026-04-16T10:00:00Z",
                "2026-04-16T10:00:00Z",
            ),
        )
        .expect("insert first title");

        let second = conn.execute(
            "INSERT INTO resources (id, title, notes, resource_type, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5)",
            (
                "resource-2",
                "my book",
                "ebook",
                "2026-04-16T11:00:00Z",
                "2026-04-16T11:00:00Z",
            ),
        );
        assert!(second.is_err(), "second title with different case should fail");
    }
}
