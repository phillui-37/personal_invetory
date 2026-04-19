# Phase 5 — Device Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote device management to a first-class feature — HTTP API (list/current/register/delink) plus a Flutter dashboard card showing all registered devices with per-device resource location counts.

**Architecture:** Hexagonal — Domain `Device` struct + `DeviceRepository` trait (in `backend/domain`) → `SqliteDeviceRepository` (rusqlite, two new migrations) + `AdapterBundle` → `DeviceService` (business rules: delink-self guard, hostname fallback) → axum handlers → Flutter `DeviceBloc` + `DeviceManagementScreen` card in `EcosystemScreen`.

**Tech Stack:** Rust/axum/rusqlite (backend), Flutter 3 / BLoC / http (frontend), `hostname = "0.3"` crate for auto-detect.

---

## Design Spec
`docs/superpowers/specs/2026-04-19-phase5-device-management-design.md`

## Pre-Phase Baseline
Backend: 251 tests green. Frontend: 159 tests green.

---

## File Map

### New files
| File | Responsibility |
|---|---|
| `backend/domain/src/device.rs` | `Device` struct + `DeviceRepository` trait |
| `backend/infrastructure/migrations/0017_alter_devices_add_name.sql` | ADD COLUMN device_name |
| `backend/infrastructure/migrations/0018_create_device_location_view.sql` | v_device_location_counts VIEW |
| `backend/infrastructure/src/sqlite/device.rs` | `SqliteDeviceRepository` (rusqlite) |
| `backend/services/src/device.rs` | `DeviceService` + `DeviceInfo` |
| `backend/adapters/src/device_handler.rs` | 4 axum handlers + DTOs |
| `frontend/lib/models/device.dart` | `Device` Dart model (Equatable + fromJson) |
| `frontend/lib/repositories/device_repository.dart` | abstract `DeviceRepository` |
| `frontend/lib/repositories/http_device_repository.dart` | `HttpDeviceRepository` |
| `frontend/lib/blocs/device/device_bloc.dart` | `DeviceBloc` events/states/handler |
| `frontend/lib/screens/device_management_screen.dart` | Device dashboard screen |
| `frontend/test/blocs/device/device_bloc_test.dart` | BLoC unit tests |
| `frontend/test/screens/device_management_screen_test.dart` | Widget tests |

### Modified files
| File | Change |
|---|---|
| `backend/domain/src/lib.rs` | `pub mod device;` |
| `backend/infrastructure/src/sqlite/mod.rs` | `pub mod device;` + SQLITE_MIGRATIONS `[&str; 18]` |
| `backend/infrastructure/src/lib.rs` | re-export `SqliteDeviceRepository` |
| `backend/infrastructure/src/factory.rs` | `AdapterBundle.device_repo` field + wiring |
| `backend/app/Cargo.toml` | add `hostname = "0.3"` |
| `backend/app/src/config.rs` | `device_id: String` (required) + `device_name: Option<String>` |
| `backend/app/src/runtime.rs` | wire `DeviceService`, update `test_config()` |
| `backend/services/src/lib.rs` | `mod device; pub use device::{DeviceInfo, DeviceService};` |
| `backend/adapters/src/lib.rs` | `mod device_handler; pub use device_handler::...` |
| `backend/adapters/src/state.rs` | `device_service: Option<Arc<DeviceService>>` + `with_device_service()` |
| `backend/adapters/src/routes.rs` | 4 device routes |
| `backend/adapters/src/openapi.rs` | device paths + schemas |
| `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs` | device repo integration tests |
| `backend/services/tests/services_tdd.rs` | `DeviceService` unit tests |
| `backend/adapters/tests/handlers_tdd.rs` | device handler integration tests |
| `frontend/test/support/fake_repositories.dart` | `FakeDeviceRepository` |
| `frontend/lib/screens/ecosystem_screen.dart` | Device Management card |
| `frontend/lib/main.dart` | `DeviceBloc` provider + `HttpDeviceRepository` |

---

## Task P5-A: Domain — Device entity and DeviceRepository trait

**Files:**
- Create: `backend/domain/src/device.rs`
- Modify: `backend/domain/src/lib.rs`

- [ ] **Step 1: Create `backend/domain/src/device.rs`**

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::DomainError;

#[derive(Debug, Clone)]
pub struct Device {
    pub id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub linked_at: DateTime<Utc>,
    pub delinked_at: Option<DateTime<Utc>>,
    pub location_count: u64,
}

#[async_trait]
pub trait DeviceRepository: Send + Sync {
    /// Returns one row per unique device_id (latest binding), with location count.
    async fn all_with_counts(&self) -> Result<Vec<Device>, DomainError>;

    /// Returns the currently active binding for a device_id, or None.
    async fn active_by_device_id(
        &self,
        device_id: &str,
    ) -> Result<Option<Device>, DomainError>;

    /// Delinks any existing active binding, then inserts a new binding.
    /// Returns the newly created Device with location_count = 0.
    async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<Device, DomainError>;

    /// Sets delinked_at on the active binding for device_id.
    /// Returns NotFound if device_id unknown, Conflict if already delinked.
    async fn delink(
        &self,
        device_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), DomainError>;
}
```

- [ ] **Step 2: Add `pub mod device;` to `backend/domain/src/lib.rs`**

Add as the first line of the file:
```rust
pub mod device;
```

- [ ] **Step 3: Verify it compiles**

```bash
cd backend && cargo check -p domain
```
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add backend/domain/src/device.rs backend/domain/src/lib.rs
git commit -m "feat(domain): add Device struct and DeviceRepository trait

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-B: Infrastructure — Write failing integration test

**Files:**
- Modify: `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs`

- [ ] **Step 1: Append the failing device repo test to `sqlite_repository_adapters_tdd.rs`**

```rust
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
        assert_eq!(active.unwrap().device_name.as_deref(), Some("Home Desktop v2"));

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
```

- [ ] **Step 2: Run test to confirm it fails to compile**

```bash
cd backend && cargo test -p infrastructure sqlite_device_repository 2>&1 | head -20
```
Expected: compile error — `AdapterBundle` has no `device_repo` field.

---

## Task P5-C: Infrastructure — Migrations + SqliteDeviceRepository (Green)

**Files:**
- Create: `backend/infrastructure/migrations/0017_alter_devices_add_name.sql`
- Create: `backend/infrastructure/migrations/0018_create_device_location_view.sql`
- Create: `backend/infrastructure/src/sqlite/device.rs`
- Modify: `backend/infrastructure/src/sqlite/mod.rs`
- Modify: `backend/infrastructure/src/lib.rs`
- Modify: `backend/infrastructure/src/factory.rs`

- [ ] **Step 1: Create migration 0017**

`backend/infrastructure/migrations/0017_alter_devices_add_name.sql`:
```sql
ALTER TABLE devices ADD COLUMN device_name TEXT;
```

- [ ] **Step 2: Create migration 0018**

`backend/infrastructure/migrations/0018_create_device_location_view.sql`:
```sql
CREATE VIEW v_device_location_counts AS
SELECT
    d.device_id,
    d.device_name,
    COUNT(rl.id) AS location_count
FROM devices d
LEFT JOIN resource_locations rl ON rl.device_id = d.device_id
WHERE d.id = (
    SELECT d2.id
    FROM devices d2
    WHERE d2.device_id = d.device_id
    ORDER BY d2.linked_at DESC
    LIMIT 1
)
GROUP BY d.device_id, d.device_name;
```

- [ ] **Step 3: Create `backend/infrastructure/src/sqlite/device.rs`**

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use uuid::Uuid;

use domain::device::{Device, DeviceRepository};
use domain::DomainError;

use super::{map_sqlite_error, parse_timestamp_for_row, SharedSqliteConnection};

pub struct SqliteDeviceRepository {
    conn: SharedSqliteConnection,
}

impl SqliteDeviceRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

fn row_to_device(row: &rusqlite::Row<'_>) -> rusqlite::Result<Device> {
    let linked_at_str: String = row.get(3)?;
    let delinked_at_str: Option<String> = row.get(4)?;
    let location_count: i64 = row.get(5)?;
    Ok(Device {
        id: row.get(0)?,
        device_id: row.get(1)?,
        device_name: row.get(2)?,
        linked_at: parse_timestamp_for_row(linked_at_str)?,
        delinked_at: delinked_at_str
            .map(parse_timestamp_for_row)
            .transpose()?,
        location_count: location_count as u64,
    })
}

#[async_trait]
impl DeviceRepository for SqliteDeviceRepository {
    async fn all_with_counts(&self) -> Result<Vec<Device>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT d.id, d.device_id, d.device_name, d.linked_at, d.delinked_at,
                        COALESCE(v.location_count, 0)
                 FROM devices d
                 LEFT JOIN v_device_location_counts v ON v.device_id = d.device_id
                 WHERE d.id = (
                     SELECT d2.id FROM devices d2
                     WHERE d2.device_id = d.device_id
                     ORDER BY d2.linked_at DESC LIMIT 1
                 )
                 ORDER BY d.linked_at DESC",
            )
            .map_err(map_sqlite_error)?;
        let rows = stmt
            .query_map([], row_to_device)
            .map_err(map_sqlite_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite_error)
    }

    async fn active_by_device_id(
        &self,
        device_id: &str,
    ) -> Result<Option<Device>, DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT d.id, d.device_id, d.device_name, d.linked_at, d.delinked_at,
                    COALESCE(v.location_count, 0)
             FROM devices d
             LEFT JOIN v_device_location_counts v ON v.device_id = d.device_id
             WHERE d.device_id = ?1 AND d.delinked_at IS NULL
             ORDER BY d.linked_at DESC LIMIT 1",
            rusqlite::params![device_id],
            row_to_device,
        )
        .optional()
        .map_err(map_sqlite_error)
    }

    async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<Device, DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let id = Uuid::new_v4().to_string();

        // Delink any existing active binding
        conn.execute(
            "UPDATE devices SET delinked_at = ?1
             WHERE device_id = ?2 AND delinked_at IS NULL",
            rusqlite::params![now_str, device_id],
        )
        .map_err(map_sqlite_error)?;

        // Insert new binding (owner_id = device_id: legacy field, same value)
        conn.execute(
            "INSERT INTO devices (id, device_id, owner_id, linked_at, device_name)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, device_id, device_id, now_str, device_name],
        )
        .map_err(map_sqlite_error)?;

        Ok(Device {
            id,
            device_id: device_id.to_string(),
            device_name: device_name.map(str::to_string),
            linked_at: now,
            delinked_at: None,
            location_count: 0,
        })
    }

    async fn delink(
        &self,
        device_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM devices WHERE device_id = ?1",
                rusqlite::params![device_id],
                |row| row.get(0),
            )
            .map_err(map_sqlite_error)?;
        if total == 0 {
            return Err(DomainError::NotFound(format!(
                "device '{device_id}' not found"
            )));
        }

        let active: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM devices
                 WHERE device_id = ?1 AND delinked_at IS NULL",
                rusqlite::params![device_id],
                |row| row.get(0),
            )
            .map_err(map_sqlite_error)?;
        if active == 0 {
            return Err(DomainError::Conflict(format!(
                "device '{device_id}' is already delinked"
            )));
        }

        let at_str = at.to_rfc3339();
        conn.execute(
            "UPDATE devices SET delinked_at = ?1
             WHERE device_id = ?2 AND delinked_at IS NULL",
            rusqlite::params![at_str, device_id],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }
}
```

- [ ] **Step 4: Update `backend/infrastructure/src/sqlite/mod.rs`**

Add `pub mod device;` with the other modules (alphabetical order after `chapter_check`):
```rust
pub mod chapter_check;
pub mod dedup;
pub mod device;       // ← add this line
pub mod ebook_meta;
```

Change the migrations array size from 16 to 18 and add the two new entries:
```rust
const SQLITE_MIGRATIONS: [&str; 18] = [
    include_str!("../../migrations/0001_create_resources.sql"),
    include_str!("../../migrations/0002_create_ebook_metas.sql"),
    include_str!("../../migrations/0003_create_web_reader_metas.sql"),
    include_str!("../../migrations/0004_create_resource_locations.sql"),
    include_str!("../../migrations/0005_create_devices.sql"),
    include_str!("../../migrations/0006_alter_web_reader_metas_add_check_fields.sql"),
    include_str!("../../migrations/0007_create_chapter_checks.sql"),
    include_str!("../../migrations/0008_create_site_configs.sql"),
    include_str!("../../migrations/0009_create_notifications.sql"),
    include_str!("../../migrations/0010_create_image_metas.sql"),
    include_str!("../../migrations/0011_create_video_metas.sql"),
    include_str!("../../migrations/0012_create_game_metas.sql"),
    include_str!("../../migrations/0013_create_vault_config.sql"),
    include_str!("../../migrations/0014_create_credentials.sql"),
    include_str!("../../migrations/0015_create_sync_jobs.sql"),
    include_str!("../../migrations/0016_create_dedup_warnings.sql"),
    include_str!("../../migrations/0017_alter_devices_add_name.sql"),
    include_str!("../../migrations/0018_create_device_location_view.sql"),
];
```

- [ ] **Step 5: Add re-export to `backend/infrastructure/src/lib.rs`**

Find the `pub use factory::{...};` line and after the existing re-exports, add:
```rust
pub use sqlite::device::SqliteDeviceRepository;
```

- [ ] **Step 6: Update `backend/infrastructure/src/factory.rs`**

Add `device_repo` field to `AdapterBundle`. Find the struct definition and add the field:
```rust
pub device_repo: Arc<dyn domain::device::DeviceRepository>,
```

In `AdapterFactory::from_url`, construct and assign the device repo. Find where other sqlite repos are built (e.g. `sync_job_repo`) and add:
```rust
let device_repo: Arc<dyn domain::device::DeviceRepository> =
    Arc::new(sqlite::device::SqliteDeviceRepository::new(shared.clone()));
```

Then add it to the `AdapterBundle { ... }` struct literal:
```rust
device_repo,
```

- [ ] **Step 7: Run the failing test to confirm it now passes**

```bash
cd backend && cargo test -p infrastructure sqlite_device_repository -- --nocapture
```
Expected: PASS.

- [ ] **Step 8: Run full backend test suite**

```bash
cd backend && cargo test --workspace 2>&1 | tail -20
```
Expected: all tests pass (≥251).

- [ ] **Step 9: Commit**

```bash
git add backend/infrastructure/migrations/0017_alter_devices_add_name.sql \
        backend/infrastructure/migrations/0018_create_device_location_view.sql \
        backend/infrastructure/src/sqlite/device.rs \
        backend/infrastructure/src/sqlite/mod.rs \
        backend/infrastructure/src/lib.rs \
        backend/infrastructure/src/factory.rs \
        backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs
git commit -m "feat(infra): SqliteDeviceRepository with migrations 0017+0018

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-D: Config — add DEVICE_ID and DEVICE_NAME

**Files:**
- Modify: `backend/app/Cargo.toml`
- Modify: `backend/app/src/config.rs`

- [ ] **Step 1: Write failing config tests**

In `backend/app/src/config.rs`, inside the `#[cfg(test)]` block, append these two tests to the existing test module:

```rust
#[test]
fn config_requires_device_id() {
    let map = base_map();
    let err = AppConfig::from_map(&map).expect_err("DEVICE_ID should be required");
    assert_eq!(err, ConfigError::MissingRequiredVar("DEVICE_ID"));
}

#[test]
fn config_reads_device_id_and_optional_device_name() {
    let mut map = base_map();
    map.insert(String::from("DEVICE_ID"), String::from("desktop-home"));
    map.insert(String::from("DEVICE_NAME"), String::from("Home Desktop"));
    let config = AppConfig::from_map(&map).expect("config loads");
    assert_eq!(config.device_id, "desktop-home");
    assert_eq!(config.device_name.as_deref(), Some("Home Desktop"));
}

#[test]
fn config_treats_blank_device_name_as_none() {
    let mut map = base_map();
    map.insert(String::from("DEVICE_ID"), String::from("desktop-home"));
    map.insert(String::from("DEVICE_NAME"), String::from("  "));
    let config = AppConfig::from_map(&map).expect("config loads");
    assert_eq!(config.device_name, None);
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cd backend && cargo test -p app config 2>&1 | tail -20
```
Expected: compile errors (fields not yet on struct).

- [ ] **Step 3: Add `hostname` crate to `backend/app/Cargo.toml`**

In `[dependencies]`:
```toml
hostname = "0.3"
```

- [ ] **Step 4: Add `device_id` and `device_name` to `AppConfig`**

In the `AppConfig` struct, add:
```rust
pub device_id: String,
pub device_name: Option<String>,
```

In `from_map`, add the parsing logic (before `Ok(Self { ... })`):
```rust
let device_id = map
    .get("DEVICE_ID")
    .cloned()
    .ok_or(ConfigError::MissingRequiredVar("DEVICE_ID"))?;
let device_name = map
    .get("DEVICE_NAME")
    .map(|v| v.trim().to_string())
    .filter(|v| !v.is_empty());
```

In the `Ok(Self { ... })` struct literal, add:
```rust
device_id,
device_name,
```

- [ ] **Step 5: Update `base_map()` in the test module to always include DEVICE_ID**

Change `base_map()` to:
```rust
fn base_map() -> BTreeMap<String, String> {
    BTreeMap::from([
        (String::from("DATABASE_URL"), String::from("sqlite://./inventory.db")),
        (String::from("DEVICE_ID"), String::from("test-device")),
    ])
}
```

Also update `config_applies_defaults_for_optional_values` to assert on the new fields:
```rust
assert_eq!(config.device_id, "test-device");
assert_eq!(config.device_name, None);
```

- [ ] **Step 6: Run config tests**

```bash
cd backend && cargo test -p app config 2>&1 | tail -20
```
Expected: all config tests pass.

- [ ] **Step 7: Commit**

```bash
git add backend/app/Cargo.toml backend/app/src/config.rs
git commit -m "feat(config): add DEVICE_ID (required) and DEVICE_NAME (optional)

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-E: Services — Write failing DeviceService test

**Files:**
- Modify: `backend/services/tests/services_tdd.rs`

- [ ] **Step 1: Append failing DeviceService tests**

```rust
// ── DeviceService ──────────────────────────────────────────────

use domain::device::{Device, DeviceRepository};

struct FakeDeviceRepo {
    devices: std::sync::Mutex<Vec<Device>>,
}

impl FakeDeviceRepo {
    fn empty() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            devices: std::sync::Mutex::new(vec![]),
        })
    }
}

#[async_trait::async_trait]
impl DeviceRepository for FakeDeviceRepo {
    async fn all_with_counts(&self) -> Result<Vec<Device>, domain::DomainError> {
        Ok(self.devices.lock().unwrap().clone())
    }
    async fn active_by_device_id(
        &self,
        device_id: &str,
    ) -> Result<Option<Device>, domain::DomainError> {
        let found = self
            .devices
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.device_id == device_id && d.delinked_at.is_none())
            .cloned();
        Ok(found)
    }
    async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<Device, domain::DomainError> {
        let d = Device {
            id: uuid::Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            device_name: device_name.map(str::to_string),
            linked_at: chrono::Utc::now(),
            delinked_at: None,
            location_count: 0,
        };
        self.devices.lock().unwrap().push(d.clone());
        Ok(d)
    }
    async fn delink(
        &self,
        device_id: &str,
        at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), domain::DomainError> {
        let mut devices = self.devices.lock().unwrap();
        let total = devices.iter().filter(|d| d.device_id == device_id).count();
        if total == 0 {
            return Err(domain::DomainError::NotFound(format!("{device_id} not found")));
        }
        let active = devices
            .iter_mut()
            .find(|d| d.device_id == device_id && d.delinked_at.is_none());
        match active {
            None => Err(domain::DomainError::Conflict(format!("{device_id} already delinked"))),
            Some(d) => {
                d.delinked_at = Some(at);
                Ok(())
            }
        }
    }
}

#[tokio::test]
async fn device_service_list_enriches_with_is_current() {
    let repo = FakeDeviceRepo::empty();
    repo.register("current-dev", None).await.unwrap();
    repo.register("other-dev", Some("Other")).await.unwrap();
    let svc = services::DeviceService::new(repo, "current-dev".into(), "myhostname".into());
    let list = svc.list().await.expect("list");
    let current = list.iter().find(|d| d.device_id == "current-dev").unwrap();
    let other = list.iter().find(|d| d.device_id == "other-dev").unwrap();
    assert!(current.is_current);
    assert!(!other.is_current);
    // hostname fallback for current device when device_name is None
    assert_eq!(current.device_name, "myhostname");
    // device_id fallback for other devices when device_name is None
    assert_eq!(other.device_name, "Other");
}

#[tokio::test]
async fn device_service_current_returns_not_found_when_unregistered() {
    let repo = FakeDeviceRepo::empty();
    let svc = services::DeviceService::new(repo, "missing".into(), "h".into());
    let err = svc.current().await.expect_err("should be NotFound");
    assert!(matches!(err, domain::DomainError::NotFound(_)));
}

#[tokio::test]
async fn device_service_register_returns_device_info() {
    let repo = FakeDeviceRepo::empty();
    let svc = services::DeviceService::new(repo, "dev1".into(), "hostname".into());
    let info = svc.register("dev1", Some("My PC")).await.expect("register");
    assert_eq!(info.device_id, "dev1");
    assert_eq!(info.device_name, "My PC");
    assert!(info.is_current);
}

#[tokio::test]
async fn device_service_delink_self_returns_validation_error() {
    let repo = FakeDeviceRepo::empty();
    repo.register("dev1", None).await.unwrap();
    let svc = services::DeviceService::new(repo, "dev1".into(), "h".into());
    let err = svc.delink("dev1").await.expect_err("cannot delink self");
    assert!(matches!(err, domain::DomainError::ValidationError(_)));
}

#[tokio::test]
async fn device_service_delink_other_device_succeeds() {
    let repo = FakeDeviceRepo::empty();
    repo.register("dev1", None).await.unwrap();
    repo.register("dev2", None).await.unwrap();
    let svc = services::DeviceService::new(repo, "dev1".into(), "h".into());
    svc.delink("dev2").await.expect("delink other device");
}
```

- [ ] **Step 2: Run to confirm Red**

```bash
cd backend && cargo test -p services device_service 2>&1 | head -20
```
Expected: compile error — `DeviceService` not found.

---

## Task P5-F: Services — DeviceService implementation (Green)

**Files:**
- Create: `backend/services/src/device.rs`
- Modify: `backend/services/src/lib.rs`

- [ ] **Step 1: Create `backend/services/src/device.rs`**

```rust
use std::sync::Arc;

use chrono::Utc;

use domain::device::{Device, DeviceRepository};
use domain::DomainError;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub device_id: String,
    /// Always non-empty: DB name, or DEVICE_NAME env, or hostname (current device),
    /// or device_id (other devices).
    pub device_name: String,
    pub linked_at: chrono::DateTime<Utc>,
    pub delinked_at: Option<chrono::DateTime<Utc>>,
    pub location_count: u64,
    pub is_current: bool,
}

pub struct DeviceService {
    repo: Arc<dyn DeviceRepository>,
    current_device_id: String,
    /// Effective display-name fallback for the current device (DEVICE_NAME or hostname).
    fallback_hostname: String,
}

impl DeviceService {
    pub fn new(
        repo: Arc<dyn DeviceRepository>,
        current_device_id: String,
        fallback_hostname: String,
    ) -> Self {
        Self {
            repo,
            current_device_id,
            fallback_hostname,
        }
    }

    fn to_info(&self, device: Device) -> DeviceInfo {
        let is_current = device.device_id == self.current_device_id;
        let device_name = device.device_name.clone().unwrap_or_else(|| {
            if is_current {
                self.fallback_hostname.clone()
            } else {
                device.device_id.clone()
            }
        });
        DeviceInfo {
            id: device.id,
            device_id: device.device_id,
            device_name,
            linked_at: device.linked_at,
            delinked_at: device.delinked_at,
            location_count: device.location_count,
            is_current,
        }
    }

    pub async fn list(&self) -> Result<Vec<DeviceInfo>, DomainError> {
        let devices = self.repo.all_with_counts().await?;
        Ok(devices.into_iter().map(|d| self.to_info(d)).collect())
    }

    pub async fn current(&self) -> Result<DeviceInfo, DomainError> {
        self.repo
            .active_by_device_id(&self.current_device_id)
            .await?
            .map(|d| self.to_info(d))
            .ok_or_else(|| DomainError::NotFound("current device not registered".into()))
    }

    pub async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<DeviceInfo, DomainError> {
        let device = self.repo.register(device_id, device_name).await?;
        Ok(self.to_info(device))
    }

    pub async fn delink(&self, device_id: &str) -> Result<(), DomainError> {
        if device_id == self.current_device_id {
            return Err(DomainError::ValidationError(
                "cannot delink the current device".into(),
            ));
        }
        self.repo.delink(device_id, Utc::now()).await
    }
}
```

- [ ] **Step 2: Add exports to `backend/services/src/lib.rs`**

Add after the existing `mod vault;` line:
```rust
mod device;
```

Add to the `pub use` section:
```rust
pub use device::{DeviceInfo, DeviceService};
```

- [ ] **Step 3: Run service tests**

```bash
cd backend && cargo test -p services device_service 2>&1 | tail -10
```
Expected: 5 tests pass.

- [ ] **Step 4: Run full backend tests**

```bash
cd backend && cargo test --workspace 2>&1 | tail -10
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add backend/services/src/device.rs backend/services/src/lib.rs \
        backend/services/tests/services_tdd.rs
git commit -m "feat(services): add DeviceService with delink-self guard and hostname fallback

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-G: Adapters — Write failing handler tests

**Files:**
- Modify: `backend/adapters/tests/handlers_tdd.rs`

- [ ] **Step 1: Append device handler tests to `handlers_tdd.rs`**

```rust
#[tokio::test]
async fn device_handlers_return_503_when_service_not_configured() {
    let openapi = r#"{"paths":{"/api/v1/system/health":{}}}"#;
    let app = app_with_openapi(openapi);

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/devices")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list.status(), StatusCode::SERVICE_UNAVAILABLE);

    let current = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/devices/current")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(current.status(), StatusCode::SERVICE_UNAVAILABLE);

    let register = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/register")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"device_id": "desktop-home"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(register.status(), StatusCode::SERVICE_UNAVAILABLE);

    let delink = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/devices/desktop-home/delink")
                .header("authorization", "Bearer secret-key")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delink.status(), StatusCode::SERVICE_UNAVAILABLE);
}
```

- [ ] **Step 2: Run to confirm Red**

```bash
cd backend && cargo test -p adapters device_handlers 2>&1 | head -20
```
Expected: compile error — routes for `/api/v1/devices` not registered.

---

## Task P5-H: Adapters — Device handlers, state, routes, openapi (Green)

**Files:**
- Create: `backend/adapters/src/device_handler.rs`
- Modify: `backend/adapters/src/lib.rs`
- Modify: `backend/adapters/src/state.rs`
- Modify: `backend/adapters/src/routes.rs`
- Modify: `backend/adapters/src/openapi.rs`

- [ ] **Step 1: Create `backend/adapters/src/device_handler.rs`**

```rust
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceResponse {
    pub id: String,
    pub device_id: String,
    pub device_name: String,
    pub linked_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delinked_at: Option<String>,
    pub location_count: u64,
    pub is_current: bool,
}

impl From<services::DeviceInfo> for DeviceResponse {
    fn from(d: services::DeviceInfo) -> Self {
        Self {
            id: d.id,
            device_id: d.device_id,
            device_name: d.device_name,
            linked_at: d.linked_at.to_rfc3339(),
            delinked_at: d.delinked_at.map(|t| t.to_rfc3339()),
            location_count: d.location_count,
            is_current: d.is_current,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterDeviceRequest {
    pub device_id: String,
    pub device_name: Option<String>,
}

fn device_svc(state: &AppState) -> Result<&Arc<services::DeviceService>, ApiError> {
    state
        .device_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("device service not configured"))
}

/// List all registered devices (active and delinked) with location counts.
#[utoipa::path(
    get,
    path = "/api/v1/devices",
    responses(
        (status = 200, description = "Device list", body = Vec<DeviceResponse>),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn list_devices(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = device_svc(&state)?;
    let devices = svc.list().await.map_err(ApiError::from)?;
    Ok(Json(
        devices
            .into_iter()
            .map(DeviceResponse::from)
            .collect::<Vec<_>>(),
    ))
}

/// Return the current backend device (identified by DEVICE_ID env var).
#[utoipa::path(
    get,
    path = "/api/v1/devices/current",
    responses(
        (status = 200, description = "Current device", body = DeviceResponse),
        (status = 404, description = "Current device not registered"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn current_device(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = device_svc(&state)?;
    let device = svc.current().await.map_err(ApiError::from)?;
    Ok(Json(DeviceResponse::from(device)))
}

/// Register a device. Re-registering delinks the old binding and creates a new one.
#[utoipa::path(
    post,
    path = "/api/v1/devices/register",
    request_body = RegisterDeviceRequest,
    responses(
        (status = 201, description = "Device registered", body = DeviceResponse),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn register_device(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterDeviceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = device_svc(&state)?;
    let info = svc
        .register(&body.device_id, body.device_name.as_deref())
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(DeviceResponse::from(info))))
}

/// Delink a device by its device_id. Cannot delink the current device.
#[utoipa::path(
    post,
    path = "/api/v1/devices/{device_id}/delink",
    params(("device_id" = String, Path, description = "Device ID to delink")),
    responses(
        (status = 200, description = "Device delinked"),
        (status = 404, description = "Device not found"),
        (status = 409, description = "Already delinked"),
        (status = 422, description = "Cannot delink current device"),
        (status = 503, description = "Service unavailable"),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn delink_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let svc = device_svc(&state)?;
    svc.delink(&device_id).await.map_err(ApiError::from)?;
    Ok(StatusCode::OK)
}
```

- [ ] **Step 2: Add `mod device_handler;` to `backend/adapters/src/lib.rs`**

After `mod dedup_handler;`:
```rust
mod device_handler;
```

Also add to the `pub use` section if you want to export the handler functions (not required, routes.rs accesses them via `crate::device_handler::`).

- [ ] **Step 3: Add `device_service` to `backend/adapters/src/state.rs`**

In the `AppState` struct, add after `sync_service`:
```rust
pub device_service: Option<Arc<services::DeviceService>>,
```

Add a `with_device_service` builder method next to `with_sync_service`:
```rust
pub fn with_device_service(mut self, svc: Arc<services::DeviceService>) -> Self {
    self.device_service = Some(svc);
    self
}
```

In `AppState::for_tests`, add:
```rust
device_service: None,
```

In `AppState::new` (the big constructor), add:
```rust
device_service: None,
```

- [ ] **Step 4: Add device routes to `backend/adapters/src/routes.rs`**

Add imports at the top:
```rust
use crate::device_handler::{current_device, delink_device, list_devices, register_device};
```

Add routes (after vault routes, before or after sync routes):
```rust
.route("/api/v1/devices", get(list_devices))
.route("/api/v1/devices/current", get(current_device))
.route("/api/v1/devices/register", post(register_device))
.route("/api/v1/devices/:device_id/delink", post(delink_device))
```

- [ ] **Step 5: Update `backend/adapters/src/openapi.rs`**

Add import:
```rust
use crate::device_handler::{DeviceResponse, RegisterDeviceRequest};
```

In the `#[openapi(paths(...))]` list, add:
```rust
crate::device_handler::list_devices,
crate::device_handler::current_device,
crate::device_handler::register_device,
crate::device_handler::delink_device,
```

In the `schemas(...)` list, add:
```rust
DeviceResponse,
RegisterDeviceRequest,
```

- [ ] **Step 6: Run handler tests**

```bash
cd backend && cargo test -p adapters device_handlers 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 7: Run full backend test suite**

```bash
cd backend && cargo test --workspace 2>&1 | tail -10
```
Expected: all tests pass.

- [ ] **Step 8: Commit**

```bash
git add backend/adapters/src/device_handler.rs \
        backend/adapters/src/lib.rs \
        backend/adapters/src/state.rs \
        backend/adapters/src/routes.rs \
        backend/adapters/src/openapi.rs \
        backend/adapters/tests/handlers_tdd.rs
git commit -m "feat(adapters): device handlers, routes, OpenAPI

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-I: App — Wire DeviceService in runtime

**Files:**
- Modify: `backend/app/src/runtime.rs`

- [ ] **Step 1: Add `hostname` and `DeviceService` imports to `runtime.rs`**

In the `use services::{ ... }` import, add `DeviceService`:
```rust
use services::{..., DeviceService};
```

- [ ] **Step 2: Wire `DeviceService` in `build_app_router`**

After the `sync_service` block, add:
```rust
let hostname_str = hostname::get()
    .ok()
    .and_then(|h| h.into_string().ok())
    .unwrap_or_else(|| config.device_id.clone());
let effective_hostname = config.device_name.clone().unwrap_or(hostname_str);
let device_service = Arc::new(DeviceService::new(
    bundle.device_repo.clone(),
    config.device_id.clone(),
    effective_hostname,
));
state = state.with_device_service(device_service);
```

- [ ] **Step 3: Update `test_config()` to include `device_id`**

Add the two new fields to the struct literal:
```rust
fn test_config() -> AppConfig {
    AppConfig {
        database_url: "sqlite://:memory:".to_string(),
        api_key: Some("secret".to_string()),
        host: "127.0.0.1".to_string(),
        port: 8080,
        plugins_config: "plugins.toml".to_string(),
        chromium_path: None,
        fcm_service_account: None,
        scheduler_enabled: false,
        device_id: "test-device".to_string(),
        device_name: None,
    }
}
```

- [ ] **Step 4: Run full backend suite**

```bash
cd backend && cargo test --workspace 2>&1 | tail -10
```
Expected: all tests pass (≥256: previous 251 + 5 new service tests).

- [ ] **Step 5: Commit**

```bash
git add backend/app/src/runtime.rs
git commit -m "feat(app): wire DeviceService in runtime with hostname fallback

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-J: Flutter — Device model

**Files:**
- Create: `frontend/lib/models/device.dart`

- [ ] **Step 1: Write the failing model test**

Create `frontend/test/models/device_test.dart`:
```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/device.dart';

void main() {
  group('Device.fromJson', () {
    test('parses active device', () {
      final json = {
        'id': 'uuid-1',
        'device_id': 'desktop-home',
        'device_name': 'Home Desktop',
        'linked_at': '2024-01-01T00:00:00.000Z',
        'location_count': 3,
        'is_current': true,
      };
      final device = Device.fromJson(json);
      expect(device.id, 'uuid-1');
      expect(device.deviceId, 'desktop-home');
      expect(device.deviceName, 'Home Desktop');
      expect(device.locationCount, 3);
      expect(device.isCurrent, isTrue);
      expect(device.isActive, isTrue);
    });

    test('parses delinked device', () {
      final json = {
        'id': 'uuid-2',
        'device_id': 'laptop-work',
        'device_name': 'Work Laptop',
        'linked_at': '2024-01-01T00:00:00.000Z',
        'delinked_at': '2024-06-01T00:00:00.000Z',
        'location_count': 0,
        'is_current': false,
      };
      final device = Device.fromJson(json);
      expect(device.delinkedAt, isNotNull);
      expect(device.isActive, isFalse);
      expect(device.isCurrent, isFalse);
    });

    test('Equatable props', () {
      final a = Device.fromJson({
        'id': 'u1',
        'device_id': 'dev',
        'device_name': 'Dev',
        'linked_at': '2024-01-01T00:00:00.000Z',
        'location_count': 0,
        'is_current': false,
      });
      final b = Device.fromJson({
        'id': 'u1',
        'device_id': 'dev',
        'device_name': 'Dev',
        'linked_at': '2024-01-01T00:00:00.000Z',
        'location_count': 0,
        'is_current': false,
      });
      expect(a, equals(b));
    });
  });
}
```

- [ ] **Step 2: Run to confirm Red**

```bash
cd frontend && flutter test test/models/device_test.dart 2>&1 | tail -10
```
Expected: error — `device.dart` not found.

- [ ] **Step 3: Create `frontend/lib/models/device.dart`**

```dart
import 'package:equatable/equatable.dart';

class Device extends Equatable {
  const Device({
    required this.id,
    required this.deviceId,
    required this.deviceName,
    required this.linkedAt,
    this.delinkedAt,
    required this.locationCount,
    required this.isCurrent,
  });

  final String id;
  final String deviceId;
  final String deviceName;
  final DateTime linkedAt;
  final DateTime? delinkedAt;
  final int locationCount;
  final bool isCurrent;

  bool get isActive => delinkedAt == null;

  factory Device.fromJson(Map<String, dynamic> json) => Device(
        id: json['id'] as String,
        deviceId: json['device_id'] as String,
        deviceName: json['device_name'] as String,
        linkedAt: DateTime.parse(json['linked_at'] as String),
        delinkedAt: json['delinked_at'] != null
            ? DateTime.parse(json['delinked_at'] as String)
            : null,
        locationCount: json['location_count'] as int,
        isCurrent: json['is_current'] as bool,
      );

  @override
  List<Object?> get props => [
        id,
        deviceId,
        deviceName,
        linkedAt,
        delinkedAt,
        locationCount,
        isCurrent,
      ];
}
```

- [ ] **Step 4: Run model tests**

```bash
cd frontend && flutter test test/models/device_test.dart 2>&1 | tail -5
```
Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add frontend/lib/models/device.dart frontend/test/models/device_test.dart
git commit -m "feat(flutter): Device model with fromJson and Equatable

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-K: Flutter — Repository layer

**Files:**
- Create: `frontend/lib/repositories/device_repository.dart`
- Create: `frontend/lib/repositories/http_device_repository.dart`
- Modify: `frontend/test/support/fake_repositories.dart`

- [ ] **Step 1: Create `frontend/lib/repositories/device_repository.dart`**

```dart
import '../models/device.dart';
import '../models/failures.dart';
import '../models/result.dart';

abstract class DeviceRepository {
  Future<Result<List<Device>, AppFailure>> listDevices();
  Future<Result<Device, AppFailure>> currentDevice();
  Future<Result<Device, AppFailure>> registerDevice({
    required String deviceId,
    String? deviceName,
  });
  Future<Result<void, AppFailure>> delinkDevice(String deviceId);
}
```

- [ ] **Step 2: Create `frontend/lib/repositories/http_device_repository.dart`**

```dart
import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/device.dart';
import '../models/failures.dart';
import '../models/result.dart';
import 'device_repository.dart';

class HttpDeviceRepository implements DeviceRepository {
  HttpDeviceRepository({required this.config, http.Client? client})
      : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
        'Authorization': config.authorizationHeader,
        'Content-Type': 'application/json',
      };

  @override
  Future<Result<List<Device>, AppFailure>> listDevices() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/devices');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final list = jsonDecode(response.body) as List<dynamic>;
        return Success(
          list
              .map((e) => Device.fromJson(e as Map<String, dynamic>))
              .toList(),
        );
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<Device, AppFailure>> currentDevice() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/devices/current');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(
          Device.fromJson(jsonDecode(response.body) as Map<String, dynamic>),
        );
      }
      if (response.statusCode == 404) {
        return const Failure(NotFoundFailure('current device'));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<Device, AppFailure>> registerDevice({
    required String deviceId,
    String? deviceName,
  }) async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/devices/register');
      final body = <String, dynamic>{'device_id': deviceId};
      if (deviceName != null) body['device_name'] = deviceName;
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(body),
      );
      if (response.statusCode == 201) {
        return Success(
          Device.fromJson(jsonDecode(response.body) as Map<String, dynamic>),
        );
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> delinkDevice(String deviceId) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/devices/${Uri.encodeComponent(deviceId)}/delink',
      );
      final response = await _client.post(uri, headers: _headers, body: '{}');
      if (response.statusCode == 200) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }
}
```

- [ ] **Step 3: Add `FakeDeviceRepository` to `frontend/test/support/fake_repositories.dart`**

Add the import at the top:
```dart
import 'package:personal_inventory_frontend/models/device.dart';
import 'package:personal_inventory_frontend/repositories/device_repository.dart';
```

Append the class at the end of the file:
```dart
class FakeDeviceRepository implements DeviceRepository {
  FakeDeviceRepository({
    this.listResult = const Success([]),
    Result<Device, AppFailure>? currentResult,
    Result<Device, AppFailure>? registerResult,
    this.delinkResult = const Success(null),
  })  : currentResult = currentResult ??
            const Failure(NotFoundFailure('current device')),
        registerResult = registerResult ??
            const Failure(ServerFailure(500));

  Result<List<Device>, AppFailure> listResult;
  Result<Device, AppFailure> currentResult;
  Result<Device, AppFailure> registerResult;
  Result<void, AppFailure> delinkResult;

  int listCalls = 0;
  int currentCalls = 0;
  int registerCalls = 0;
  String? lastDelinkId;
  String? lastRegisterDeviceId;
  String? lastRegisterDeviceName;

  @override
  Future<Result<List<Device>, AppFailure>> listDevices() async {
    listCalls += 1;
    return listResult;
  }

  @override
  Future<Result<Device, AppFailure>> currentDevice() async {
    currentCalls += 1;
    return currentResult;
  }

  @override
  Future<Result<Device, AppFailure>> registerDevice({
    required String deviceId,
    String? deviceName,
  }) async {
    registerCalls += 1;
    lastRegisterDeviceId = deviceId;
    lastRegisterDeviceName = deviceName;
    return registerResult;
  }

  @override
  Future<Result<void, AppFailure>> delinkDevice(String deviceId) async {
    lastDelinkId = deviceId;
    return delinkResult;
  }
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd frontend && flutter analyze lib/repositories/device_repository.dart \
    lib/repositories/http_device_repository.dart \
    test/support/fake_repositories.dart 2>&1 | tail -10
```
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add frontend/lib/repositories/device_repository.dart \
        frontend/lib/repositories/http_device_repository.dart \
        frontend/test/support/fake_repositories.dart
git commit -m "feat(flutter): DeviceRepository + HttpDeviceRepository + FakeDeviceRepository

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-L: Flutter — Write failing DeviceBloc test

**Files:**
- Create: `frontend/test/blocs/device/device_bloc_test.dart`

- [ ] **Step 1: Create test directory and file**

```bash
mkdir -p frontend/test/blocs/device
```

Create `frontend/test/blocs/device/device_bloc_test.dart`:
```dart
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/device/device_bloc.dart';
import 'package:personal_inventory_frontend/models/device.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/device_repository.dart';

final _device1 = Device(
  id: 'u1',
  deviceId: 'desktop-home',
  deviceName: 'Home Desktop',
  linkedAt: DateTime.utc(2024),
  locationCount: 2,
  isCurrent: true,
);

final _device2 = Device(
  id: 'u2',
  deviceId: 'laptop-work',
  deviceName: 'laptop-work',
  linkedAt: DateTime.utc(2024),
  locationCount: 0,
  isCurrent: false,
);

void main() {
  group('DeviceBloc', () {
    blocTest<DeviceBloc, DeviceState>(
      'emits loading then list on LoadDevices success',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onList: () async => Success([_device1, _device2]),
      )),
      act: (bloc) => bloc.add(const LoadDevices()),
      expect: () => [
        const DeviceLoading(),
        DeviceListLoaded([_device1, _device2]),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then error on LoadDevices failure',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onList: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const LoadDevices()),
      expect: () => const [
        DeviceLoading(),
        DeviceError(NetworkFailure('offline')),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then current loaded on LoadCurrentDevice success',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onCurrent: () async => Success(_device1),
      )),
      act: (bloc) => bloc.add(const LoadCurrentDevice()),
      expect: () => [
        const DeviceLoading(),
        DeviceCurrentLoaded(_device1),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then error on LoadCurrentDevice not-found',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onCurrent: () async =>
            const Failure(NotFoundFailure('current device')),
      )),
      act: (bloc) => bloc.add(const LoadCurrentDevice()),
      expect: () => const [
        DeviceLoading(),
        DeviceError(NotFoundFailure('current device')),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then operation success on RegisterDevice',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onRegister: (_, __) async => Success(_device1),
      )),
      act: (bloc) => bloc.add(
        const RegisterDevice(deviceId: 'desktop-home', deviceName: 'Home'),
      ),
      expect: () => const [
        DeviceLoading(),
        DeviceOperationSuccess(DeviceOperationType.register),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then error on RegisterDevice failure',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onRegister: (_, __) async => const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const RegisterDevice(deviceId: 'x')),
      expect: () => const [
        DeviceLoading(),
        DeviceError(ServerFailure(500)),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then operation success on DelinkDevice',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onDelink: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const DelinkDevice('laptop-work')),
      expect: () => const [
        DeviceLoading(),
        DeviceOperationSuccess(DeviceOperationType.delink),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then error on DelinkDevice failure',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onDelink: (_) async => const Failure(ServerFailure(422)),
      )),
      act: (bloc) => bloc.add(const DelinkDevice('desktop-home')),
      expect: () => const [
        DeviceLoading(),
        DeviceError(ServerFailure(422)),
      ],
    );
  });
}

final class _FakeDeviceRepo implements DeviceRepository {
  _FakeDeviceRepo({
    this.onList,
    this.onCurrent,
    this.onRegister,
    this.onDelink,
  });

  final Future<Result<List<Device>, AppFailure>> Function()? onList;
  final Future<Result<Device, AppFailure>> Function()? onCurrent;
  final Future<Result<Device, AppFailure>> Function(
    String deviceId,
    String? deviceName,
  )? onRegister;
  final Future<Result<void, AppFailure>> Function(String)? onDelink;

  @override
  Future<Result<List<Device>, AppFailure>> listDevices() =>
      onList?.call() ?? Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<Device, AppFailure>> currentDevice() =>
      onCurrent?.call() ?? Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<Device, AppFailure>> registerDevice({
    required String deviceId,
    String? deviceName,
  }) =>
      onRegister?.call(deviceId, deviceName) ??
      Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<void, AppFailure>> delinkDevice(String deviceId) =>
      onDelink?.call(deviceId) ??
      Future.value(const Failure(ServerFailure(500)));
}
```

- [ ] **Step 2: Run to confirm Red**

```bash
cd frontend && flutter test test/blocs/device/device_bloc_test.dart 2>&1 | head -15
```
Expected: error — `device_bloc.dart` not found.

---

## Task P5-M: Flutter — DeviceBloc implementation (Green)

**Files:**
- Create: `frontend/lib/blocs/device/device_bloc.dart`

- [ ] **Step 1: Create `frontend/lib/blocs/device/device_bloc.dart`**

```dart
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/device.dart';
import '../../models/failures.dart';
import '../../models/result.dart';
import '../../repositories/device_repository.dart';

// ── Events ──────────────────────────────────────────────────────

sealed class DeviceEvent extends Equatable {
  const DeviceEvent();
}

final class LoadDevices extends DeviceEvent {
  const LoadDevices();
  @override
  List<Object?> get props => [];
}

final class LoadCurrentDevice extends DeviceEvent {
  const LoadCurrentDevice();
  @override
  List<Object?> get props => [];
}

final class RegisterDevice extends DeviceEvent {
  const RegisterDevice({required this.deviceId, this.deviceName});
  final String deviceId;
  final String? deviceName;
  @override
  List<Object?> get props => [deviceId, deviceName];
}

final class DelinkDevice extends DeviceEvent {
  const DelinkDevice(this.deviceId);
  final String deviceId;
  @override
  List<Object?> get props => [deviceId];
}

// ── States ──────────────────────────────────────────────────────

sealed class DeviceState extends Equatable {
  const DeviceState();
}

final class DeviceInitial extends DeviceState {
  const DeviceInitial();
  @override
  List<Object?> get props => [];
}

final class DeviceLoading extends DeviceState {
  const DeviceLoading();
  @override
  List<Object?> get props => [];
}

final class DeviceListLoaded extends DeviceState {
  const DeviceListLoaded(this.devices);
  final List<Device> devices;
  @override
  List<Object?> get props => [devices];
}

final class DeviceCurrentLoaded extends DeviceState {
  const DeviceCurrentLoaded(this.device);
  final Device device;
  @override
  List<Object?> get props => [device];
}

final class DeviceOperationSuccess extends DeviceState {
  const DeviceOperationSuccess(this.type);
  final DeviceOperationType type;
  @override
  List<Object?> get props => [type];
}

final class DeviceError extends DeviceState {
  const DeviceError(this.failure);
  final AppFailure failure;
  @override
  List<Object?> get props => [failure];
}

enum DeviceOperationType { register, delink }

// ── Bloc ─────────────────────────────────────────────────────────

class DeviceBloc extends Bloc<DeviceEvent, DeviceState> {
  DeviceBloc(this._repo) : super(const DeviceInitial()) {
    on<LoadDevices>(_onLoadDevices);
    on<LoadCurrentDevice>(_onLoadCurrentDevice);
    on<RegisterDevice>(_onRegisterDevice);
    on<DelinkDevice>(_onDelinkDevice);
  }

  final DeviceRepository _repo;

  Future<void> _onLoadDevices(
    LoadDevices event,
    Emitter<DeviceState> emit,
  ) async {
    emit(const DeviceLoading());
    final result = await _repo.listDevices();
    result.when(
      success: (devices) => emit(DeviceListLoaded(devices)),
      failure: (f) => emit(DeviceError(f)),
    );
  }

  Future<void> _onLoadCurrentDevice(
    LoadCurrentDevice event,
    Emitter<DeviceState> emit,
  ) async {
    emit(const DeviceLoading());
    final result = await _repo.currentDevice();
    result.when(
      success: (device) => emit(DeviceCurrentLoaded(device)),
      failure: (f) => emit(DeviceError(f)),
    );
  }

  Future<void> _onRegisterDevice(
    RegisterDevice event,
    Emitter<DeviceState> emit,
  ) async {
    emit(const DeviceLoading());
    final result = await _repo.registerDevice(
      deviceId: event.deviceId,
      deviceName: event.deviceName,
    );
    result.when(
      success: (_) =>
          emit(const DeviceOperationSuccess(DeviceOperationType.register)),
      failure: (f) => emit(DeviceError(f)),
    );
  }

  Future<void> _onDelinkDevice(
    DelinkDevice event,
    Emitter<DeviceState> emit,
  ) async {
    emit(const DeviceLoading());
    final result = await _repo.delinkDevice(event.deviceId);
    result.when(
      success: (_) =>
          emit(const DeviceOperationSuccess(DeviceOperationType.delink)),
      failure: (f) => emit(DeviceError(f)),
    );
  }
}
```

- [ ] **Step 2: Run DeviceBloc tests**

```bash
cd frontend && flutter test test/blocs/device/device_bloc_test.dart 2>&1 | tail -10
```
Expected: 8 tests pass.

- [ ] **Step 3: Commit**

```bash
git add frontend/lib/blocs/device/device_bloc.dart \
        frontend/test/blocs/device/device_bloc_test.dart
git commit -m "feat(flutter): DeviceBloc with 8 passing tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-N: Flutter — Write failing widget test

**Files:**
- Create: `frontend/test/screens/device_management_screen_test.dart`

- [ ] **Step 1: Create the widget test**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/device/device_bloc.dart';
import 'package:personal_inventory_frontend/models/device.dart';
import 'package:personal_inventory_frontend/screens/device_management_screen.dart';

import '../support/fake_repositories.dart';

Widget _buildScreen(FakeDeviceRepository repo) {
  return MaterialApp(
    home: BlocProvider<DeviceBloc>(
      create: (_) => DeviceBloc(repo),
      child: const DeviceManagementScreen(),
    ),
  );
}

final _currentDevice = Device(
  id: 'u1',
  deviceId: 'desktop-home',
  deviceName: 'Home Desktop',
  linkedAt: DateTime.utc(2024),
  locationCount: 3,
  isCurrent: true,
);

final _otherDevice = Device(
  id: 'u2',
  deviceId: 'laptop-work',
  deviceName: 'laptop-work',
  linkedAt: DateTime.utc(2024),
  locationCount: 0,
  isCurrent: false,
);

void main() {
  group('DeviceManagementScreen', () {
    testWidgets('shows loading indicator then device list', (tester) async {
      final repo = FakeDeviceRepository(
        listResult: Success([_currentDevice, _otherDevice]),
      );
      await tester.pumpWidget(_buildScreen(repo));
      await tester.tap(find.text('Refresh'));
      await tester.pump();
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      await tester.pumpAndSettle();
      expect(find.text('Home Desktop'), findsOneWidget);
      expect(find.text('laptop-work'), findsOneWidget);
    });

    testWidgets('shows delink button only on active non-current devices',
        (tester) async {
      final repo = FakeDeviceRepository(
        listResult: Success([_currentDevice, _otherDevice]),
      );
      await tester.pumpWidget(_buildScreen(repo));
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      // Delink button appears for the non-current active device
      expect(find.text('Delink'), findsOneWidget);
    });

    testWidgets('shows error message on failure', (tester) async {
      final repo = FakeDeviceRepository(
        listResult: const Failure(NetworkFailure('offline')),
      );
      await tester.pumpWidget(_buildScreen(repo));
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(find.textContaining('offline'), findsOneWidget);
    });

    testWidgets('shows register form with device_id field', (tester) async {
      await tester.pumpWidget(_buildScreen(FakeDeviceRepository()));
      expect(find.text('Register Device'), findsOneWidget);
      expect(find.byType(TextField), findsWidgets);
    });
  });
}
```

- [ ] **Step 2: Run to confirm Red**

```bash
cd frontend && flutter test test/screens/device_management_screen_test.dart 2>&1 | head -15
```
Expected: error — `device_management_screen.dart` not found.

---

## Task P5-O: Flutter — DeviceManagementScreen + navigation (Green)

**Files:**
- Create: `frontend/lib/screens/device_management_screen.dart`
- Modify: `frontend/lib/screens/ecosystem_screen.dart`
- Modify: `frontend/lib/main.dart`

- [ ] **Step 1: Create `frontend/lib/screens/device_management_screen.dart`**

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/device/device_bloc.dart';
import '../models/device.dart';

class DeviceManagementScreen extends StatelessWidget {
  const DeviceManagementScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Device Management')),
      body: BlocBuilder<DeviceBloc, DeviceState>(
        builder: (context, state) {
          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              _RegisterDeviceForm(),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () =>
                    context.read<DeviceBloc>().add(const LoadDevices()),
                child: const Text('Refresh'),
              ),
              const SizedBox(height: 16),
              if (state is DeviceLoading)
                const Center(child: CircularProgressIndicator())
              else if (state is DeviceListLoaded)
                ...state.devices.map((d) => _DeviceTile(device: d))
              else if (state is DeviceOperationSuccess) ...[
                const Text('Operation succeeded'),
              ] else if (state is DeviceError)
                Text(
                  'Error: ${state.failure}',
                  style: const TextStyle(color: Colors.red),
                ),
            ],
          );
        },
      ),
    );
  }
}

class _RegisterDeviceForm extends StatefulWidget {
  @override
  State<_RegisterDeviceForm> createState() => _RegisterDeviceFormState();
}

class _RegisterDeviceFormState extends State<_RegisterDeviceForm> {
  final _deviceIdCtrl = TextEditingController();
  final _deviceNameCtrl = TextEditingController();

  @override
  void dispose() {
    _deviceIdCtrl.dispose();
    _deviceNameCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Register Device',
          style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 8),
        TextField(
          controller: _deviceIdCtrl,
          decoration: const InputDecoration(
            labelText: 'Device ID',
            hintText: 'e.g. desktop-home',
          ),
        ),
        const SizedBox(height: 8),
        TextField(
          controller: _deviceNameCtrl,
          decoration: const InputDecoration(
            labelText: 'Display Name (optional)',
          ),
        ),
        const SizedBox(height: 8),
        ElevatedButton(
          onPressed: () {
            final id = _deviceIdCtrl.text.trim();
            if (id.isEmpty) return;
            final name = _deviceNameCtrl.text.trim();
            context.read<DeviceBloc>().add(
                  RegisterDevice(
                    deviceId: id,
                    deviceName: name.isEmpty ? null : name,
                  ),
                );
          },
          child: const Text('Register'),
        ),
      ],
    );
  }
}

class _DeviceTile extends StatelessWidget {
  const _DeviceTile({required this.device});
  final Device device;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: ListTile(
        leading: Icon(
          device.isActive ? Icons.computer : Icons.computer_outlined,
          color: device.isCurrent ? Colors.blue : null,
        ),
        title: Text(device.deviceName),
        subtitle: Text(
          '${device.locationCount} location(s)'
          '${device.isCurrent ? ' · current' : ''}'
          '${!device.isActive ? ' · delinked' : ''}',
        ),
        trailing: (device.isActive && !device.isCurrent)
            ? TextButton(
                onPressed: () => context
                    .read<DeviceBloc>()
                    .add(DelinkDevice(device.deviceId)),
                child: const Text('Delink'),
              )
            : null,
      ),
    );
  }
}
```

- [ ] **Step 2: Add Device Management card to `frontend/lib/screens/ecosystem_screen.dart`**

Add import at top:
```dart
import '../blocs/device/device_bloc.dart';
import 'device_management_screen.dart';
```

In the `ListView` children, append a new `Card` (after the Dedup Review card):
```dart
Card(
  child: ListTile(
    leading: const Icon(Icons.devices),
    title: const Text('Device Management'),
    subtitle: const Text('Manage registered devices and their locations'),
    trailing: const Icon(Icons.chevron_right),
    onTap: () => Navigator.push(
      context,
      MaterialPageRoute<void>(
        builder: (_) => BlocProvider.value(
          value: context.read<DeviceBloc>(),
          child: const DeviceManagementScreen(),
        ),
      ),
    ),
  ),
),
```

- [ ] **Step 3: Update `frontend/lib/main.dart`**

Add imports:
```dart
import 'blocs/device/device_bloc.dart';
import 'repositories/http_device_repository.dart';
```

In the body of the `build` method (alongside `vaultRepository`, `dedupRepository`, etc.):
```dart
final deviceRepository = HttpDeviceRepository(config: config);
```

In the `MultiBlocProvider` `providers` list, add:
```dart
BlocProvider<DeviceBloc>(create: (_) => DeviceBloc(deviceRepository)),
```

- [ ] **Step 4: Run widget tests**

```bash
cd frontend && flutter test test/screens/device_management_screen_test.dart 2>&1 | tail -10
```
Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add frontend/lib/screens/device_management_screen.dart \
        frontend/lib/screens/ecosystem_screen.dart \
        frontend/lib/main.dart \
        frontend/test/screens/device_management_screen_test.dart
git commit -m "feat(flutter): DeviceManagementScreen with register/delink/list

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task P5-P: Final Verification and Documentation

- [ ] **Step 1: Run full backend test suite**

```bash
cd backend && cargo test --workspace 2>&1 | tail -20
```
Expected: ≥256 tests pass, 0 failures.

- [ ] **Step 2: Run full Flutter test suite**

```bash
cd frontend && flutter test 2>&1 | tail -20
```
Expected: ≥171 tests pass (159 + 3 model + 8 bloc + 4 widget = 12 new minimum), 0 failures.

- [ ] **Step 3: Update `CONTEXT.md`**

Append to `CONTEXT.md`:
```
## Phase 5 — Device Management (completed)

Implemented device management as a first-class feature:
- Backend: `DeviceRepository` trait, `SqliteDeviceRepository` (migrations 0017+0018),
  `DeviceService` (delink-self guard, hostname/DEVICE_NAME fallback),
  4 HTTP endpoints (`GET /api/v1/devices`, `GET /api/v1/devices/current`,
  `POST /api/v1/devices/register`, `POST /api/v1/devices/:device_id/delink`).
- Config: `DEVICE_ID` (required), `DEVICE_NAME` (optional, blank → None).
- Frontend: `Device` model, `DeviceRepository` + `HttpDeviceRepository`,
  `DeviceBloc`, `DeviceManagementScreen`, card in `EcosystemScreen`.
- `ValidationError` (delink self) → HTTP 422 per existing `ApiError` mapping.
- `owner_id` legacy field: set to `device_id` value on all new inserts.
```

- [ ] **Step 4: Final commit**

```bash
git add CONTEXT.md
git commit -m "docs: update CONTEXT.md for Phase 5 completion

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Implementation Notes

- `DomainError::ValidationError` maps to HTTP **422** (not 400) per `adapters/src/error.rs`.
- `owner_id` column is a legacy field (from Phase 1). For all new inserts in Phase 5, set `owner_id = device_id`.
- `v_device_location_counts` returns one row per unique `device_id` (latest binding), so the `all_with_counts` query is safe to LEFT JOIN against it without multiplying rows.
- The `fallback_hostname` passed to `DeviceService::new` is: `config.device_name.unwrap_or(hostname::get())`. This is computed in `runtime.rs`, not inside the service.
- The `hostname` crate is added only to `backend/app/Cargo.toml` (not to `services`), since hostname resolution is an infrastructure concern wired at the app layer.
