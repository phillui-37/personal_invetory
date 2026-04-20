# Phase 7 — PostgreSQL, MOBI/AZW3, WebView, Batch Metadata, Android Build

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the remaining hard requirements from TODO.md:
1. Full PostgreSQL adapter (all 15 repos) so the system works with `postgres://` URLs.
2. DB portability — reliable SQLite ↔ PG data transfer.
3. MOBI/AZW3 metadata extraction (backend plugin + Flutter extractor).
4. Real WebView progress tracking (replace `TODO(D8)` placeholder).
5. Batch metadata update + copy operations (backend + frontend).
6. Android build setup.

**Architecture:** Same Hexagonal + Clean pattern used throughout. PG repos use `sqlx` (async) instead of `rusqlite` (sync). All other work extends existing seams.

**Tech Stack:**
- Backend: `sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "macros", "chrono", "uuid"] }`
- Frontend (WebView): `webview_flutter = "^4.0"` (already in flutter ecosystem)
- Frontend (MOBI): pure-Dart PalmDoc header parser (no native bridge needed for basic metadata)
- Android: `minSdkVersion 21`, `targetSdkVersion 34`

---

## Pre-Phase Baseline
Backend: 318 tests green. Frontend: 204 tests green.

---

## File Map

### New files

#### Backend
| File | Responsibility |
|---|---|
| `backend/infrastructure/src/postgres/pool.rs` | `PgPool` factory via sqlx |
| `backend/infrastructure/src/postgres/migrations.rs` | sqlx migrate runner for PG |
| `backend/infrastructure/src/postgres/resource.rs` | Real sqlx PG implementation (replaces stub) |
| `backend/infrastructure/src/postgres/ebook_meta.rs` | Real sqlx PG implementation |
| `backend/infrastructure/src/postgres/web_reader_meta.rs` | Real sqlx PG implementation |
| `backend/infrastructure/src/postgres/location.rs` | Real sqlx PG implementation |
| `backend/infrastructure/src/postgres/chapter_check.rs` | Real sqlx PG implementation |
| `backend/infrastructure/src/postgres/notification.rs` | Real sqlx PG implementation |
| `backend/infrastructure/src/postgres/image_meta.rs` | sqlx PG (new file, no stub exists) |
| `backend/infrastructure/src/postgres/video_meta.rs` | sqlx PG (new file) |
| `backend/infrastructure/src/postgres/game_meta.rs` | sqlx PG (new file) |
| `backend/infrastructure/src/postgres/vault.rs` | sqlx PG (new file) |
| `backend/infrastructure/src/postgres/sync_job.rs` | sqlx PG (new file) |
| `backend/infrastructure/src/postgres/dedup.rs` | sqlx PG (new file) |
| `backend/infrastructure/src/postgres/device.rs` | sqlx PG (new file) |
| `backend/infrastructure/src/postgres/progress.rs` | sqlx PG (new file) |
| `backend/infrastructure/src/postgres/tag.rs` | sqlx PG (new file) |
| `backend/infrastructure/migrations_pg/` | PG-compatible SQL migration files (0001–0021) |
| `backend/plugins/src/mobi.rs` | PalmDoc/MOBI header parser (title, author, publisher) |

#### Frontend
| File | Responsibility |
|---|---|
| `frontend/lib/plugins/mobi_metadata_extractor_impl.dart` | Real MOBI/AZW3 header parser in Dart |
| `frontend/lib/screens/batch_metadata_screen.dart` | Batch update + copy metadata UI |
| `frontend/lib/blocs/batch/batch_bloc.dart` | BatchBloc (SelectAll, UpdateMetadata, CopyMetadata) |
| `frontend/lib/repositories/batch_repository.dart` | Abstract batch ops interface |
| `frontend/lib/repositories/http_batch_repository.dart` | HTTP implementation |
| `frontend/lib/models/batch.dart` | BatchUpdateInput, BatchCopyInput models |
| `frontend/android/` | Gradle config, AndroidManifest.xml (internet, storage permissions) |
| `frontend/test/plugins/mobi_extractor_impl_test.dart` | Unit tests for real MOBI extractor |
| `frontend/test/blocs/batch/batch_bloc_test.dart` | BLoC unit tests |
| `frontend/test/screens/batch_metadata_screen_test.dart` | Widget tests |

### Modified files

#### Backend
| File | Change |
|---|---|
| `backend/infrastructure/Cargo.toml` | Add `sqlx` with postgres features |
| `backend/infrastructure/src/postgres/mod.rs` | Export all new modules + `PgPool` |
| `backend/infrastructure/src/factory.rs` | Wire PG bundle when `postgres://` URL |
| `backend/infrastructure/src/portability.rs` | Expand `CanonicalResourceSnapshot` (image/video/game/progress/tags/devices) + PG import/export |
| `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs` | Portability tests with new snapshot fields |
| `backend/adapters/src/ebook.rs` | `PATCH /api/v1/inventory/ebooks/batch-update` handler |
| `backend/adapters/src/routes.rs` | Register batch-update + batch-copy-meta routes for all types |
| `backend/services/src/ebook.rs` | `batch_update` + `batch_copy_meta` service methods |
| `backend/plugins/src/lib.rs` | Export `mobi` module |

#### Frontend
| File | Change |
|---|---|
| `frontend/lib/plugins/mobi_metadata_extractor.dart` | Delegate to real impl instead of always returning UnsupportedFailure |
| `frontend/lib/widgets/web_reader_progress_tracker.dart` | Replace TODO(D8) with real `webview_flutter` WebViewWidget |
| `frontend/lib/screens/resource_list_screen.dart` | Floating action button / menu linking to BatchMetadataScreen |
| `frontend/lib/main.dart` | Provide BatchBloc |
| `frontend/pubspec.yaml` | Add `webview_flutter: ^4.0` |
| `frontend/android/app/build.gradle` | `minSdkVersion 21`, `targetSdkVersion 34` |
| `frontend/android/app/src/main/AndroidManifest.xml` | INTERNET + READ_EXTERNAL_STORAGE permissions |

---

## DB Schema (PG migrations)

PG migrations are SQL files in `backend/infrastructure/migrations_pg/` numbered 0001–0021, matching the SQLite migration set but using PG-compatible syntax (e.g. `BOOLEAN` instead of `INTEGER`, `TEXT` same, no `AUTOINCREMENT`). Key differences:
- Use `CREATE INDEX IF NOT EXISTS` (same as SQLite)
- `CHECK` constraints same
- PG uses standard SQL `CREATE TABLE IF NOT EXISTS`
- Add `DEFAULT gen_random_uuid()` where applicable as fallback

---

## API Surface (new endpoints)

### Batch metadata update
```
PATCH /api/v1/inventory/:type/batch-update
Body: { "ids": ["id1","id2"], "fields": { "author": "...", "language": "..." } }
Response: { "updated": 2, "failed": [] }
```

### Batch metadata copy
```
POST /api/v1/inventory/:type/batch-copy-meta
Body: { "source_id": "id1", "target_ids": ["id2","id3"], "fields": ["author","publisher"] }
Response: { "updated": 2, "failed": [] }
```

---

## Task P7-A: PostgreSQL infrastructure setup

**Goal:** Add `sqlx` PG dependency; create async pool factory + PG migrations runner. No repo implementations yet.

**Files:**
- Modify: `backend/infrastructure/Cargo.toml`
- Create: `backend/infrastructure/src/postgres/pool.rs`
- Create: `backend/infrastructure/src/postgres/migrations.rs`
- Create: `backend/infrastructure/migrations_pg/` (0001–0021 SQL files)
- Modify: `backend/infrastructure/src/postgres/mod.rs`
- Modify: `backend/infrastructure/src/lib.rs`

- [ ] **Step 1: Add sqlx dependency**

In `backend/infrastructure/Cargo.toml`:
```toml
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "macros", "chrono", "uuid"], optional = true }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

[features]
postgres = ["dep:sqlx"]
firebase = [...]
```

- [ ] **Step 2: Create `backend/infrastructure/src/postgres/pool.rs`**

```rust
use sqlx::PgPool;
use domain::DomainError;

pub async fn open_pg_pool(database_url: &str) -> Result<PgPool, DomainError> {
    PgPool::connect(database_url)
        .await
        .map_err(|e| DomainError::InternalError(format!("PG pool connect failed: {e}")))
}
```

- [ ] **Step 3: Create `backend/infrastructure/src/postgres/migrations.rs`**

```rust
use sqlx::PgPool;
use domain::DomainError;

pub async fn run_migrations(pool: &PgPool) -> Result<(), DomainError> {
    sqlx::migrate!("./migrations_pg")
        .run(pool)
        .await
        .map_err(|e| DomainError::InternalError(format!("PG migration failed: {e}")))
}
```

- [ ] **Step 4: Create `backend/infrastructure/migrations_pg/` SQL files (0001–0021)**

Mirror each SQLite migration file, adapted for PG dialect:
- Remove `AUTOINCREMENT` (not valid in PG — use default TEXT PKs as-is)
- `INTEGER` → `BIGINT` for counts where appropriate; TEXT PKs stay TEXT
- Add `EXTENSION` for `gen_random_uuid()` if needed (optional, PKs are app-generated UUIDs)
- Each file should be idempotent (`CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`)

- [ ] **Step 5: Update `backend/infrastructure/src/postgres/mod.rs`**

```rust
pub mod migrations;
pub mod pool;
pub mod chapter_check;
pub mod ebook_meta;
pub mod location;
pub mod notification;
pub mod resource;
pub mod web_reader_meta;
// new (P7-B onwards):
pub mod image_meta;
pub mod video_meta;
pub mod game_meta;
pub mod vault;
pub mod sync_job;
pub mod dedup;
pub mod device;
pub mod progress;
pub mod tag;
```

- [ ] **Step 6: Write compile-test**

Add test in `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs`:
```rust
#[test]
fn pg_pool_module_compiles() {
    // static check that the module tree is valid
    let _ = std::any::type_name::<infrastructure::postgres::pool::open_pg_pool>();
}
```

- [ ] **RED → GREEN → REFACTOR**

---

## Task P7-B: PG repos — resource, ebook_meta, web_reader_meta, location

**Goal:** Replace 4 existing stubs with real sqlx async implementations.

**Files:**
- Rewrite: `backend/infrastructure/src/postgres/resource.rs`
- Rewrite: `backend/infrastructure/src/postgres/ebook_meta.rs`
- Rewrite: `backend/infrastructure/src/postgres/web_reader_meta.rs`
- Rewrite: `backend/infrastructure/src/postgres/location.rs`

**Pattern per repo (example: resource):**
```rust
use sqlx::PgPool;
use async_trait::async_trait;
use domain::{DomainError, Resource, NewResource, ResourceRepository, UpdateResource};
use uuid::Uuid;

pub struct PgResourceRepository { pool: PgPool }

impl PgResourceRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl ResourceRepository for PgResourceRepository {
    async fn list(&self) -> Result<Vec<Resource>, DomainError> {
        let rows = sqlx::query_as!(ResourceRow, "SELECT * FROM resources ORDER BY created_at DESC")
            .fetch_all(&self.pool).await.map_err(pg_err)?;
        rows.into_iter().map(try_from_row).collect()
    }
    // ... search, get_by_id, create, update, delete
}

fn pg_err(e: sqlx::Error) -> DomainError {
    DomainError::InternalError(format!("PG error: {e}"))
}
```

- [ ] **Step 1: Write TDD contract tests for PgResourceRepository**

In `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs` (or a new `pg_repository_tdd.rs`), add tests guarded by `#[cfg(feature = "postgres")]` that skip if `TEST_PG_URL` env var is absent:

```rust
#[tokio::test]
#[cfg(feature = "postgres")]
async fn pg_resource_repo_create_and_list() {
    let url = match std::env::var("TEST_PG_URL") {
        Ok(u) => u,
        Err(_) => return, // skip when no PG available
    };
    let pool = infrastructure::postgres::pool::open_pg_pool(&url).await.unwrap();
    infrastructure::postgres::migrations::run_migrations(&pool).await.unwrap();
    let repo = PgResourceRepository::new(pool);
    // create, list, assert
}
```

- [ ] **Step 2: Implement `PgResourceRepository`** — RED → GREEN
- [ ] **Step 3: Implement `PgEbookMetaRepository`** — RED → GREEN
- [ ] **Step 4: Implement `PgWebReaderMetaRepository`** — RED → GREEN
- [ ] **Step 5: Implement `PgLocationRepository`** — RED → GREEN
- [ ] **Refactor: extract shared `pg_err` helper into `postgres/error.rs`**

---

## Task P7-C: PG repos — chapter_check, notification, image_meta, video_meta, game_meta

**Goal:** Replace 2 existing stubs + implement 3 new PG repos.

**Files:**
- Rewrite: `backend/infrastructure/src/postgres/chapter_check.rs`
- Rewrite: `backend/infrastructure/src/postgres/notification.rs`
- Create: `backend/infrastructure/src/postgres/image_meta.rs`
- Create: `backend/infrastructure/src/postgres/video_meta.rs`
- Create: `backend/infrastructure/src/postgres/game_meta.rs`

Follow the same sqlx pattern as P7-B. Each repo implements its domain trait.

- [ ] Write TDD tests (skip if no `TEST_PG_URL`) for each repo
- [ ] Implement `PgChapterCheckRepository` — RED → GREEN
- [ ] Implement `PgNotificationRepository` — RED → GREEN
- [ ] Implement `PgImageMetaRepository` — RED → GREEN
- [ ] Implement `PgVideoMetaRepository` — RED → GREEN
- [ ] Implement `PgGameMetaRepository` — RED → GREEN

---

## Task P7-D: PG repos — vault, sync_job, dedup, device, progress, tag

**Goal:** Implement 6 new PG repos for Phase 4–6 data.

**Files:**
- Create: `backend/infrastructure/src/postgres/vault.rs`
- Create: `backend/infrastructure/src/postgres/sync_job.rs`
- Create: `backend/infrastructure/src/postgres/dedup.rs`
- Create: `backend/infrastructure/src/postgres/device.rs`
- Create: `backend/infrastructure/src/postgres/progress.rs`
- Create: `backend/infrastructure/src/postgres/tag.rs`

`tag.rs` implements both `TagRepository` and `ResourceTagRepository` (same as SQLite).

- [ ] Write TDD tests (skip if no `TEST_PG_URL`) for each
- [ ] Implement `PgVaultBackend` — RED → GREEN
- [ ] Implement `PgSyncJobRepository` — RED → GREEN
- [ ] Implement `PgDedupWarningRepository` — RED → GREEN
- [ ] Implement `PgDeviceRepository` — RED → GREEN
- [ ] Implement `PgProgressRepository` — RED → GREEN
- [ ] Implement `PgTagRepository` + `PgResourceTagRepository` in `tag.rs` — RED → GREEN

---

## Task P7-E: Factory PG wiring + integration smoke test

**Goal:** `AdapterFactory::from_url` with `postgres://` URL returns a fully wired `AdapterBundle` using the new PG repos.

**Files:**
- Modify: `backend/infrastructure/src/factory.rs`
- Modify: `backend/infrastructure/Cargo.toml` (gate PG imports behind feature)
- Modify: `backend/adapters/tests/handlers_tdd.rs` (optional smoke test guarded by `TEST_PG_URL`)

- [ ] **Step 1: Implement PG branch in `AdapterFactory::from_url`**

```rust
#[cfg(feature = "postgres")]
if database_url.starts_with("postgres://") {
    let pool = postgres::pool::open_pg_pool(database_url)
        .await  // factory needs to be async, or use block_on
        .map_err(...)?;
    postgres::migrations::run_migrations(&pool).await?;
    let tag_repo = Arc::new(postgres::tag::PgTagRepository::new(pool.clone()));
    return Ok(AdapterBundle {
        database: DatabaseAdapter::Postgres,
        resource_repo: Arc::new(postgres::resource::PgResourceRepository::new(pool.clone())),
        // ... all 15 repos
    });
}
```

Note: `from_url` is currently synchronous (`fn`, not `async fn`). Promote to `async fn from_url` — update all call sites in `app/src/runtime.rs` to `.await`.

- [ ] **Step 2: Update `backend/app/src/runtime.rs`** — `AdapterFactory::from_url(...).await`
- [ ] **Step 3: Smoke test** — handler test that creates bundle from `TEST_PG_URL` and calls `/api/v1/system/health`
- [ ] **RED → GREEN → REFACTOR**

---

## Task P7-F: DB portability — full SQLite ↔ PG snapshot

**Goal:** `CanonicalResourceSnapshot` covers all current tables; bidirectional export/import works for both backends.

**Files:**
- Modify: `backend/infrastructure/src/portability.rs`
- Modify: `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs`

- [ ] **Step 1: Expand `CanonicalResourceSnapshot`**

Add new row types mirroring all current tables:
```rust
pub struct ImageMetaRow { pub resource_id: String, pub file_format: Option<String>, ... }
pub struct VideoMetaRow { ... }
pub struct GameMetaRow { ... }
pub struct ProgressRow { pub resource_id: String, pub progress: f64, pub notes: Option<String>, pub updated_at: String }
pub struct TagRow { pub id: String, pub name: String, pub created_at: String }
pub struct ResourceTagRow { pub resource_id: String, pub tag_id: String }
pub struct DeviceRow { pub id: String, pub name: Option<String>, ... }
```

Update `CanonicalResourceSnapshot` to include all new `Vec<*Row>` fields.

- [ ] **Step 2: Implement SQLite full export** (read all tables → snapshot)
- [ ] **Step 3: Implement SQLite full import** (snapshot → insert all tables, idempotent)
- [ ] **Step 4: Implement PG full export** (gated `#[cfg(feature = "postgres")]`)
- [ ] **Step 5: Implement PG full import** (gated)
- [ ] **Step 6: Round-trip test** (SQLite export → PG import → PG export → assert equal; skip if no `TEST_PG_URL`)
- [ ] **RED → GREEN → REFACTOR**

---

## Task P7-G: MOBI/AZW3 backend metadata plugin

**Goal:** Parse MOBI PalmDoc headers in Rust to extract title, author, publisher. No external crate — manual byte parsing.

**Files:**
- Create: `backend/plugins/src/mobi.rs`
- Modify: `backend/plugins/src/lib.rs`

**MOBI PalmDoc header layout (offsets from file start):**
- Bytes 0–31: PalmDB name (NUL-terminated, use as title fallback)
- Bytes 76–77: Number of records (big-endian u16)
- MOBI header starts after PalmDB header (78 bytes) + record list
- EXTH header (if flag bit set in MOBI header at offset 0x80) contains title, author, publisher as tagged records

- [ ] **Step 1: TDD — write `mobi_header_parse_title` test** with a minimal synthetic MOBI byte fixture
- [ ] **Step 2: Implement `parse_mobi_metadata(bytes: &[u8]) -> MobiMetadata`**
  - `MobiMetadata { title: Option<String>, author: Option<String>, publisher: Option<String> }`
  - Parse PalmDB name as title fallback
  - Walk EXTH records if MOBI header present: record type 100=author, 101=publisher, 503=updated_title
  - Gracefully return partial results on malformed input (never panic)
- [ ] **Step 3: Expose via plugin trait** — implement `MetadataExtractor` for MOBI in plugins crate
- [ ] **RED → GREEN → REFACTOR**

---

## Task P7-H: MOBI/AZW3 frontend extractor (Dart)

**Goal:** Replace `UnsupportedFailure` stub with a real Dart implementation parsing MOBI headers.

**Files:**
- Create: `frontend/lib/plugins/mobi_metadata_extractor_impl.dart`
- Modify: `frontend/lib/plugins/mobi_metadata_extractor.dart`
- Create: `frontend/test/plugins/mobi_extractor_impl_test.dart`

**Approach:** Pure Dart byte parsing using `dart:typed_data`. Mirrors the Rust logic from P7-G.

- [ ] **Step 1: Write tests in `mobi_extractor_impl_test.dart`**

```dart
test('extracts title from PalmDB name field', () async {
  // minimal 78-byte PalmDB header with "My Book" in first 32 bytes
  final bytes = _makePalmDbHeader('My Book');
  final result = await MobiMetadataExtractorImpl().extractFromBytes(bytes);
  expect((result as Success).value.title, 'My Book');
});

test('returns partial result on truncated file', () async { ... });
```

- [ ] **Step 2: Implement `MobiMetadataExtractorImpl`**
  - `extractFromBytes(Uint8List bytes) → Future<Result<ExtractedMetadata, AppFailure>>`
  - Read PalmDB name (bytes 0–31, NUL-terminated) as title fallback
  - If EXTH present: walk records for author (100), publisher (101), updated_title (503)
  - Gracefully return `Success` with partial data on malformed input
- [ ] **Step 3: Wire into `MobiMetadataExtractor.extract()`** — delegate to impl instead of returning `UnsupportedFailure`
- [ ] **RED → GREEN → REFACTOR**

---

## Task P7-I: Real WebView progress tracking

**Goal:** Replace `TODO(D8)` placeholder in `web_reader_progress_tracker.dart` with a real `webview_flutter` WebViewWidget that auto-detects URL/DOM changes and fires `onProgressUpdate`.

**Files:**
- Modify: `frontend/pubspec.yaml` (add `webview_flutter: ^4.0`)
- Modify: `frontend/lib/widgets/web_reader_progress_tracker.dart`
- Modify: `frontend/test/widgets/web_reader_progress_tracker_test.dart`

- [ ] **Step 1: Add `webview_flutter` to `pubspec.yaml`**, run `flutter pub get`
- [ ] **Step 2: Rewrite `WebReaderProgressTracker` widget**

```dart
// Replace the TODO(D8) placeholder with:
WebViewWidget(
  controller: _controller,
)
// where _controller is WebViewController configured with:
//   .setJavaScriptMode(JavaScriptMode.unrestricted)
//   .setNavigationDelegate(NavigationDelegate(
//       onPageFinished: (url) => _invokeProgressUpdate(url),
//   ))
//   .addJavaScriptChannel('ProgressBridge', onMessageReceived: _onJsMessage)
//   .loadRequest(Uri.parse(widget.url))
```

- [ ] **Step 3: Update `_invokeProgressUpdate`** to call `widget.onProgressUpdate` with the new URL as chapter signal
- [ ] **Step 4: Update tests** — mock `WebViewController` or test the JS channel callback path
- [ ] **RED → GREEN → REFACTOR**

---

## Task P7-J: Batch metadata update — backend

**Goal:** Add `PATCH /api/v1/inventory/:type/batch-update` endpoint that applies partial field updates to multiple resources of the same type.

**Files:**
- Modify: `backend/services/src/ebook.rs` (and image/video/game/web_reader)
- Create/Modify: `backend/adapters/src/batch_handler.rs`
- Modify: `backend/adapters/src/routes.rs`
- Modify: `backend/adapters/src/state.rs`
- Modify: `backend/adapters/tests/handlers_tdd.rs`

**DTOs:**
```rust
#[derive(Deserialize, ToSchema)]
pub struct BatchUpdateRequest {
    pub ids: Vec<String>,
    pub fields: serde_json::Value, // sparse update fields
}

#[derive(Serialize, ToSchema)]
pub struct BatchUpdateResponse {
    pub updated: usize,
    pub failed: Vec<BatchOpFailure>,
}
```

**Service method:**
```rust
pub async fn batch_update(&self, ids: Vec<Uuid>, fields: UpdateEbookInput) -> BatchUpdateResponse
```

- [ ] **Step 1: Write TDD handler tests** — `PATCH /api/v1/inventory/ebooks/batch-update` with valid + partially-invalid IDs
- [ ] **Step 2: Implement `batch_update` service method** for all 5 types — RED → GREEN
- [ ] **Step 3: Implement handler + route registration** for all 5 types
- [ ] **Step 4: OpenAPI annotations** for new endpoint + DTOs
- [ ] **Refactor: extract shared `BatchUpdateResponse` struct** into `adapters/src/batch_handler.rs`

---

## Task P7-K: Batch metadata copy — backend + frontend

**Goal:** `POST /api/v1/inventory/:type/batch-copy-meta` copies selected fields from one source resource to multiple targets. Flutter `BatchMetadataScreen` for both operations.

**Backend files:**
- Modify: `backend/services/src/ebook.rs` (+ others)
- Modify: `backend/adapters/src/batch_handler.rs`
- Modify: `backend/adapters/src/routes.rs`
- Modify: `backend/adapters/tests/handlers_tdd.rs`

**Frontend files:**
- Create: `frontend/lib/models/batch.dart`
- Create: `frontend/lib/repositories/batch_repository.dart`
- Create: `frontend/lib/repositories/http_batch_repository.dart`
- Create: `frontend/lib/blocs/batch/batch_bloc.dart`
- Create: `frontend/lib/screens/batch_metadata_screen.dart`
- Modify: `frontend/lib/screens/resource_list_screen.dart` (add batch FAB)
- Modify: `frontend/lib/main.dart` (provide BatchBloc)
- Create: `frontend/test/blocs/batch/batch_bloc_test.dart`
- Create: `frontend/test/screens/batch_metadata_screen_test.dart`

**Service method:**
```rust
pub async fn batch_copy_meta(
    &self,
    source_id: Uuid,
    target_ids: Vec<Uuid>,
    fields: Vec<CopyableField>,
) -> BatchUpdateResponse
```

- [ ] **Step 1: Backend TDD** — copy-meta handler tests (valid source, unknown target ID returns in `failed`)
- [ ] **Step 2: Implement `batch_copy_meta` service method** for Ebook, Image, Video, Game — RED → GREEN
- [ ] **Step 3: Register `POST /:type/batch-copy-meta` route** and handler
- [ ] **Step 4: Flutter TDD** — `BatchBloc` tests (SelectResource, SetTargets, ExecuteUpdate, ExecuteCopy states)
- [ ] **Step 5: Implement `BatchBloc` + `BatchMetadataScreen`** — RED → GREEN
- [ ] **Step 6: Wire into `ResourceListScreen`** (long-press or FAB → BatchMetadataScreen)
- [ ] **Refactor: screen test providers** — add `BatchBloc(FakeBatchRepository())` to all relevant test setups

---

## Task P7-L: Android build setup

**Goal:** Configure Flutter Android target so the app builds with `flutter build apk` on macOS.

**Files:**
- Modify: `frontend/android/app/build.gradle`
- Modify: `frontend/android/app/src/main/AndroidManifest.xml`
- Create: `docs/build-android.md` (build instructions)

- [ ] **Step 1: Update `build.gradle`**
  ```groovy
  android {
      compileSdkVersion 34
      defaultConfig {
          minSdkVersion 21
          targetSdkVersion 34
      }
  }
  ```

- [ ] **Step 2: Update `AndroidManifest.xml`**
  ```xml
  <uses-permission android:name="android.permission.INTERNET"/>
  <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE"
                   android:maxSdkVersion="32"/>
  ```
  (For API 33+, use `READ_MEDIA_*` permissions for specific file types.)

- [ ] **Step 3: Verify `webview_flutter` Android setup** — add `android:usesCleartextTraffic="true"` if needed for local HTTP

- [ ] **Step 4: Run `flutter build apk --debug`** — resolve any build errors

- [ ] **Step 5: Document** in `docs/build-android.md`: prerequisites, build commands, signing setup note

---

## Test Strategy

| Task | Backend tests | Frontend tests |
|------|--------------|----------------|
| P7-A | Compile test | — |
| P7-B | 5 PG repo tests (skip no-PG) | — |
| P7-C | 5 PG repo tests (skip no-PG) | — |
| P7-D | 6 PG repo tests (skip no-PG) | — |
| P7-E | Factory + smoke test | — |
| P7-F | Round-trip portability test | — |
| P7-G | MOBI byte fixture tests | — |
| P7-H | — | MOBI extractor unit tests |
| P7-I | — | WebView widget tests (mock controller) |
| P7-J | Handler batch-update tests | — |
| P7-K | Handler batch-copy tests | BatchBloc + screen widget tests |
| P7-L | — | `flutter build apk` succeeds |

---

## Dependency Order

```
P7-A (PG setup)
  └─ P7-B (core repos)
       └─ P7-C (more repos)
            └─ P7-D (ecosystem repos)
                 └─ P7-E (factory wiring)
                      └─ P7-F (portability)

P7-G (MOBI backend) — independent
P7-H (MOBI frontend) — independent
P7-I (WebView) — independent
P7-J (batch update backend) → P7-K (batch copy + frontend)
P7-L (Android build) — independent, can run in parallel with P7-I
```

P7-A through P7-F must run sequentially (each builds on the previous). All other tracks are independent.

---

## Key Technical Decisions

1. **`sqlx` over `tokio-postgres`**: `sqlx` provides compile-time query checking, async, and matches the codebase's async trait pattern better.
2. **`factory::from_url` becomes `async fn`**: Required because `PgPool::connect` is async. All call sites (`runtime.rs`) already use `.await`; minimal change.
3. **PG migrations in `migrations_pg/`**: Separate from SQLite `migrations/` because some syntax differs. sqlx `migrate!` macro takes a path.
4. **PG tests skip without `TEST_PG_URL`**: Avoids CI dependency on a running PG instance while still providing coverage when available.
5. **MOBI parsing pure Dart/Rust**: No external crates/packages — keeps deps minimal and the parser small (~100 lines each).
6. **`webview_flutter` v4**: Stable API with `WebViewController` builder pattern; supports Android + iOS + macOS.
7. **Batch copy is type-scoped**: Copying metadata between resource types is not supported (avoids schema mismatch complexity).
