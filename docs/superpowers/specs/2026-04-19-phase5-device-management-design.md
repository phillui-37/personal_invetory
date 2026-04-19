# Phase 5: Device Management — Design Spec

**Date**: 2026-04-19  
**Phase**: 5  
**Scope**: Device management only (focused phase)  
**Methodology**: TDD Red → Green → Refactor

---

## Problem Statement

Device identity infrastructure was established in Phase 1 (`devices` table, `DeviceBinding`, raw rusqlite helpers) but no HTTP API or frontend UX was ever built. Users cannot see which devices are registered, how many resources are located on each device, or delink obsolete devices. Phase 5 promotes device management to a full, first-class feature consistent with the hexagonal architecture used throughout the project.

---

## Architecture

Same hexagonal stack as all prior phases:

| Layer | Component |
|---|---|
| Domain | `Device` entity, `DeviceRepository` trait |
| Services | `DeviceService` (list, current, register, delink) |
| Infrastructure | `SqliteDeviceRepository` (sqlx), Migration 0017, Migration 0018 (view) |
| Adapters | 4 HTTP endpoints + utoipa annotations, axum handlers |
| Config | `DEVICE_ID` (UUID, required), `DEVICE_NAME` (optional, defaults to hostname) |
| Frontend | `DeviceBloc` + `DeviceManagementScreen` + `DeviceRepository` interface |

The existing raw rusqlite functions (`register_device_owner`, `active_owner_by_device_id`) in `backend/infrastructure/src/lib.rs` are retained for bootstrap use only. All API-facing device reads/writes go through the new sqlx repository.

---

## Data Model

### Migration 0017 — Add device_name column

```sql
ALTER TABLE devices ADD COLUMN device_name TEXT;
```

Nullable; existing rows get `NULL`. New registrations set this from the `RegisterDeviceRequest.device_name` field (or from `hostname::get()` on the backend when absent).

### Migration 0018 — Resource location count view

```sql
CREATE VIEW IF NOT EXISTS v_device_location_counts AS
SELECT
    d.device_id,
    d.device_name,
    d.linked_at,
    d.delinked_at,
    COUNT(rl.id) AS location_count
FROM devices d
LEFT JOIN resource_locations rl ON rl.device_id = d.device_id
WHERE d.id = (
    SELECT id FROM devices d2
    WHERE d2.device_id = d.device_id
    ORDER BY d2.linked_at DESC
    LIMIT 1
)
GROUP BY d.device_id;
```

Shows exactly one row per `device_id` (the latest binding — either the current active one or the most recently delinked one). Read-only. Used by `SqliteDeviceRepository::list` and `SqliteDeviceRepository::get`.

### Domain `Device` entity

```rust
pub struct Device {
    pub device_id: String,          // UUID string
    pub device_name: Option<String>,
    pub linked_at: String,          // ISO 8601
    pub delinked_at: Option<String>,
    pub location_count: u64,        // from view
}
```

`Device` is Active when `delinked_at` is `None`.

### `DeviceRepository` trait

```rust
#[async_trait]
pub trait DeviceRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Device>, DomainError>;
    async fn get(&self, device_id: &str) -> Result<Option<Device>, DomainError>;
    async fn register(&self, device_id: &str, device_name: Option<String>, linked_at: &str) -> Result<Device, DomainError>;
    async fn delink(&self, device_id: &str, delinked_at: &str) -> Result<(), DomainError>;
}
```

---

## API Endpoints

All endpoints require the standard Bearer API key.

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/devices` | List all devices (active + delinked history), each with `location_count` |
| `GET` | `/api/v1/devices/current` | Current backend device info (from `DEVICE_ID` env var) |
| `POST` | `/api/v1/devices/register` | Register a device; re-registering an existing active device updates name and `linked_at` |
| `POST` | `/api/v1/devices/:id/delink` | Soft-delink a device (sets `delinked_at = now`) |

### Request / Response DTOs

**`RegisterDeviceRequest`**
```json
{ "device_id": "uuid-string", "device_name": "Phil's MacBook" }
```
`device_name` is optional; backend defaults to `hostname::get()` if absent.

**`DeviceResponse`** (used in list and register responses)
```json
{
  "device_id": "uuid-string",
  "device_name": "Phil's MacBook",
  "linked_at": "2026-04-19T12:00:00Z",
  "delinked_at": null,
  "location_count": 42,
  "is_current": true
}
```
`is_current` is true when `device_id == DEVICE_ID` env var.

---

## Services

`DeviceService::new(repo: Arc<dyn DeviceRepository>, current_device_id: String, default_hostname: String)`

Methods:
- `list() -> Vec<DeviceResponse>` — queries repo, marks `is_current`
- `current() -> Option<DeviceResponse>` — gets the entry for `current_device_id`
- `register(device_id, device_name?) -> DeviceResponse` — upserts (insert or update name + linked_at); delinks old active binding first if device_id already active
- `delink(device_id) -> Result<(), DomainError>`:
  - Returns `DomainError::ValidationError` if `device_id == current_device_id` (can't delink self)
  - Returns `DomainError::Conflict` if device is already delinked
  - Sets `delinked_at = now()` via repo

---

## Config Changes

**`backend/app/src/config.rs`**:
- `device_id: String` — required, read from `DEVICE_ID` env var; missing → `ConfigError::MissingRequiredVar`
- `device_name: Option<String>` — optional, read from `DEVICE_NAME` env var; blank → `None`

**Runtime**: if `device_name` is `None`, fall back to `hostname::get()` at startup. Add `hostname` crate to `app/Cargo.toml`.

**`owner_id` schema field**: The `devices` table has `owner_id TEXT NOT NULL` (legacy from Phase 1). Since this is a single-user system, `owner_id` carries no semantic meaning in Phase 5. The `SqliteDeviceRepository::register` implementation passes `device_id` as the `owner_id` value for all new inserts. The `Device` domain struct and all API responses omit `owner_id`.

---

## Frontend

### Models

```dart
class Device extends Equatable {
  final String deviceId;
  final String? deviceName;
  final String linkedAt;
  final String? delinkedAt;
  final int locationCount;
  final bool isCurrent;

  bool get isActive => delinkedAt == null;
}
```

### Repository

```dart
abstract class DeviceRepository {
  Future<Result<List<Device>>> listDevices();
  Future<Result<Device>> currentDevice();
  Future<Result<Device>> registerDevice(String deviceId, {String? deviceName});
  Future<Result<void>> delinkDevice(String deviceId);
}
```

`HttpDeviceRepository` + `FakeDeviceRepository` (for tests).

### BLoC

Events: `LoadDevices`, `RegisterCurrentDevice(deviceName?)`, `DelinkDevice(deviceId)`  
States: `DeviceInitial`, `DeviceLoading`, `DevicesLoaded(devices, currentDeviceId)`, `DeviceOperationSuccess`, `DeviceError(message)`

### Screen: `DeviceManagementScreen`

Navigation: added as a card in `EcosystemScreen` (same hub pattern as Vault / Dedup / Sync).

Layout:
1. **Current Device card** (top): shows device_id, name, location count, status badge. Shows "Register This Device" button if `current()` returns null. Name field pre-filled with hostname, user can override before registering.
2. **All Devices list**: each card shows name (or device_id if null), Active/Delinked badge, location count, and a "Delink" button for active devices that are not the current device.

---

## Error Handling

| Scenario | Backend | Frontend |
|---|---|---|
| `DEVICE_ID` env var missing | Bootstrap fails with `ConfigError::MissingRequiredVar` | N/A |
| Register already-active device | Upsert (no error; updates name + linked_at) | Shows success |
| Delink current device | `DomainError::ValidationError("cannot delink current device")` → HTTP 400 | Snackbar error |
| Delink already-delinked device | `DomainError::Conflict` → HTTP 409 | Snackbar error |
| Device not found | `DomainError::NotFound` → HTTP 404 | Snackbar error |

---

## Testing Plan

### Backend (TDD Red→Green→Refactor)

**Unit — `DeviceService`** (fake repo):
- list returns devices with `is_current` flag set correctly
- register new device creates entry with hostname fallback name
- register existing active device updates name and linked_at (upsert)
- delink succeeds for a different active device
- delink self → `ValidationError`
- delink already-delinked → `Conflict`
- current() returns None when device not registered yet

**Integration — `SqliteDeviceRepository`**:
- register round-trip (insert + read back)
- register again (upsert) — verify device_name updated
- list shows active + delinked entries
- delink sets `delinked_at`
- view `v_device_location_counts` reflects correct location_count

**Handler tests** (axum test client):
- `GET /api/v1/devices` → 200 + list
- `GET /api/v1/devices/current` → 200 or 404
- `POST /api/v1/devices/register` → 201 created / 200 upsert
- `POST /api/v1/devices/:id/delink` → 200 / 400 (self) / 409 (already delinked) / 404

### Frontend (TDD)

**`DeviceBloc` unit tests**:
- `LoadDevices` → `DevicesLoaded`
- `RegisterCurrentDevice` → `DeviceOperationSuccess` then reload
- `DelinkDevice` → `DeviceOperationSuccess` then reload
- error from repo → `DeviceError`

**`DeviceManagementScreen` widget tests**:
- shows current device card
- shows "Register" button when current device absent
- shows delink button only on active non-current devices
- empty state handled

---

## References

- `CONTEXT.md` — architecture, prior phases
- `TODO.md` — original requirements (device management deferred from Phase 1)
- `AGENTS.md` — mandatory workflow
- Prior spec: `docs/superpowers/specs/2026-04-18-phase4-ecosystem-integrations-design.md`
