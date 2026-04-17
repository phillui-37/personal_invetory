# Phase 2 Tasks — Real Plugin Implementations

**Spec context:** `CONTEXT.md`, `TODO.md`  
**Approach:** Backend-first (new backend capabilities → Flutter wiring)  
**Methodology:** TDD — Red → Green → Refactor on every task  
**Scope:** Real WebChecker (headless Chromium), ebook metadata extraction (Flutter client-side), batch import endpoint, background scheduler, SSE + Firebase notifications

---

## Key Decisions (from pre-planning Q&A)

| Topic | Decision |
|---|---|
| WebChecker mechanism | Headless Chromium via `chromiumoxide` (native Rust) |
| DOM query strategy | CSS selector + XPath + regex on text — all three supported, config picks which |
| Metadata extraction | Client-side Flutter only; Rust `MetadataExtractor` trait stays stubbed |
| Ebook formats | PDF, EPUB, MOBI, AZW3 — all four; MOBI/AZW3 via best available Dart package, FFI or subprocess fallback |
| Single import | Extract → auto-fill form → user confirms → POST to existing create endpoint |
| Bulk import | Extract all → POST to new `POST /api/v1/inventory/ebooks/batch-import` endpoint |
| Chapter check schedule | Both: auto (configurable interval per resource) + manual trigger |
| New chapter notification | Store in DB + SSE push to Flutter + Firebase FCM |
| Firebase | Integrated in backend (FCM HTTP v1 API) and Flutter (`firebase_messaging`) |
| Docker | Add `chromium` to Alpine runtime image |

---

## Architecture Additions

### New backend endpoints
```
POST /api/v1/inventory/ebooks/batch-import
POST /api/v1/inventory/web-readers/:id/check
GET  /api/v1/inventory/web-readers/:id/checks
GET  /api/v1/notifications
POST /api/v1/notifications/:id/read
GET  /api/v1/notifications/stream        (SSE)
```

### New DB tables (Phase 2 migrations)
- `chapter_checks`: check history per web reader resource
- `site_configs`: per-site DOM/CSS/XPath/regex selector config + interval override
- `notifications`: chapter-detected notifications (read/unread)
- Alter `web_reader_metas`: add `check_interval_secs`, `last_checked_at`

### New Cargo features
- `real-plugins` (in `plugins` crate): enables `ChromiumWebChecker`
- `firebase` (in `infrastructure` crate): enables FCM client

### `plugins.toml` schema (example)
```toml
[web_checker]
default_interval_secs = 3600

[[web_checker.sites]]
url_pattern = "https://comic-walker\\.com/.*"
css_selector = ".episode-title"
text_regex = "第(\\d+)話"
check_interval_secs = 1800

[[web_checker.sites]]
url_pattern = "https://www\\.pixiv\\.net/.*"
xpath = "//h1[contains(@class,'title')]"
text_regex = "\\d+"
```

---

## Track C — Backend (Rust)

### C1 · Domain — Chapter Check Entities & Repository Traits
**Crate:** `domain` | **Zone:** Pure

New structs:
```rust
pub struct ChapterCheck {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub has_new_chapter: bool,
    pub latest_chapter: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

pub struct Notification {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}
```

Extend `WebReaderMeta`:
```rust
pub check_interval_secs: Option<u64>,
pub last_checked_at: Option<DateTime<Utc>>,
```

New repository traits (`#[async_trait]`, `Send + Sync`):
```rust
ChapterCheckRepository:
    create(resource_id, has_new, latest_chapter, error) -> Result<ChapterCheck, DomainError>
    list(resource_id) -> Result<Vec<ChapterCheck>, DomainError>

NotificationRepository:
    create(resource_id, message) -> Result<Notification, DomainError>
    list(unread_only: bool) -> Result<Vec<Notification>, DomainError>
    mark_read(id: Uuid) -> Result<(), DomainError>
```

**TDD:** In-memory mock implementations of both new traits; contract tests for CRUD + error cases; serde round-trip tests for new structs.

**Acceptance:** `cargo test -p domain` green. No I/O or async impls in crate.

---

### C2 · Infrastructure — DB Migrations for Phase 2
**Crate:** `infrastructure` | **Zone:** Impure

Migration files (SQLite/PG-compatible — `TEXT` PKs, no `JSONB`/`ARRAY`/`SERIAL`):

- `0006_alter_web_reader_metas_add_check_fields.sql`
  ```sql
  ALTER TABLE web_reader_metas ADD COLUMN check_interval_secs INTEGER;
  ALTER TABLE web_reader_metas ADD COLUMN last_checked_at TEXT;
  ```
- `0007_create_chapter_checks.sql`
  ```sql
  CREATE TABLE chapter_checks (
    id TEXT PRIMARY KEY,
    resource_id TEXT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    has_new_chapter INTEGER NOT NULL DEFAULT 0,
    latest_chapter TEXT,
    checked_at TEXT NOT NULL,
    error_message TEXT
  );
  CREATE INDEX chapter_checks_resource_id ON chapter_checks(resource_id);
  ```
- `0008_create_site_configs.sql`
  ```sql
  CREATE TABLE site_configs (
    id TEXT PRIMARY KEY,
    url_pattern TEXT NOT NULL,
    css_selector TEXT,
    xpath TEXT,
    text_regex TEXT,
    check_interval_secs INTEGER
  );
  ```
- `0009_create_notifications.sql`
  ```sql
  CREATE TABLE notifications (
    id TEXT PRIMARY KEY,
    resource_id TEXT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL,
    read INTEGER NOT NULL DEFAULT 0
  );
  CREATE INDEX notifications_read ON notifications(read);
  ```

**TDD:** Run all 9 migrations against in-memory SQLite; assert tables, columns, and indices exist. Regression test: all Phase 1 migration tests still pass.

**Acceptance:** Migration test green. All 9 migrations apply without error.

---

### C3 · Infrastructure — SQLite Repositories for Phase 2
**Crate:** `infrastructure` | **Zone:** Impure

Implement concrete SQLite adapters:

- `SqliteChapterCheckRepository` implementing `ChapterCheckRepository`
  - `create`: INSERT returning full row
  - `list`: SELECT ORDER BY `checked_at DESC`
- `SqliteNotificationRepository` implementing `NotificationRepository`
  - `create`: INSERT
  - `list(unread_only)`: SELECT with optional `WHERE read = 0`
  - `mark_read`: UPDATE `read = 1` WHERE id
- Extend `SqliteWebReaderMetaRepository`: include `check_interval_secs`, `last_checked_at` in upsert/get

PG scaffolds: stub impls returning `DomainError::InternalError("not implemented")` (same pattern as Phase 1).

**TDD:** Integration tests against in-memory SQLite for all methods; error contract tests (NotFound, etc.).

**Acceptance:** `cargo test -p infrastructure` green.

---

### C4 · Plugins — TOML Site Config Schema & Loader
**Crate:** `plugins` | **Zone:** Pure (data types) + Impure (file read in `app`)

Add to `plugins` crate (pure — no file I/O, just types + parser):
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct SiteConfig {
    pub url_pattern: String,
    pub css_selector: Option<String>,
    pub xpath: Option<String>,
    pub text_regex: Option<String>,
    pub check_interval_secs: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WebCheckerConfig {
    pub default_interval_secs: u64,
    pub sites: Vec<SiteConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginsToml {
    pub web_checker: Option<WebCheckerConfig>,
}
```

Helper: `fn match_site_config<'a>(url: &str, configs: &'a [SiteConfig]) -> Option<&'a SiteConfig>` — iterate configs, test `url_pattern` as regex match against `url`.

Add `toml` + `regex` to `plugins` Cargo.toml dependencies.

**TDD:**
- TOML parsing: valid full config, config with only some fields, empty config
- `match_site_config`: URL matches first applicable site, no-match returns None

**Acceptance:** `cargo test -p plugins` green. No file I/O in this crate.

---

### C5 · Plugins — Real WebChecker (chromiumoxide)
**Crate:** `plugins` | **Zone:** Impure (behind `real-plugins` feature)

Add to `plugins/Cargo.toml`:
```toml
[features]
default = []
stub-plugins = []
real-plugins = ["dep:chromiumoxide", "dep:tokio"]

[dependencies]
chromiumoxide = { version = "...", optional = true }
tokio = { version = "1", features = ["full"], optional = true }
```

`ChromiumWebChecker` (behind `#[cfg(feature = "real-plugins")]`):
```rust
pub struct ChromiumWebChecker {
    config: WebCheckerConfig,
    chromium_path: Option<String>,  // from env CHROMIUM_PATH
}

impl WebChecker for ChromiumWebChecker {
    fn check(&self, url: &str) -> Result<CheckResult, PluginError>;
}
```

Implementation steps in `check`:
1. Find matching `SiteConfig` via `match_site_config` — return `UnsupportedInput` if none.
2. Launch headless Chrome (`chromiumoxide::Browser::launch`), set `CHROMIUM_PATH` if configured.
3. Open new page, navigate to `url`, wait for `DOMContentLoaded`.
4. Apply in priority order (all configured ones):
   - CSS selector: `page.find_element(selector)?.get_inner_text()`
   - XPath: `page.evaluate("document.evaluate(...)")`
   - Text regex: `page.content()` → apply regex → capture group 1
5. Compare extracted text to previously known `latest_chapter` (passed in via `CheckInput`).
6. Return `CheckResult { has_new: bool, latest_chapter: Option<String> }`.
7. Close browser on drop.

Update `WebChecker` trait signature to accept current known chapter for comparison:
```rust
pub trait WebChecker: Send + Sync {
    fn check(&self, url: &str, known_chapter: Option<&str>) -> Result<CheckResult, PluginError>;
}
```
Update `NoOpWebChecker` accordingly.

**TDD:**
- Unit tests: `match_site_config` with various URL patterns
- Integration test with a local `axum` test server serving mock HTML — run only under `real-plugins` feature + CI flag
- `NoOpWebChecker` updated tests still pass

**Acceptance:** `cargo test -p plugins --features real-plugins` green. `cargo test -p plugins` (stub) green.

---

### C6 · App — TOML Plugin Config Loading & Wiring
**Crate:** `app` | **Zone:** Impure

In `app/src/config.rs`:
- `AppConfig` gains `chromium_path: Option<String>` (from env `CHROMIUM_PATH`)
- `AppConfig` gains `fcm_service_account: Option<String>` (from env `FCM_SERVICE_ACCOUNT_JSON`)
- `AppConfig` gains `scheduler_enabled: bool` (from env `SCHEDULER_ENABLED`, default `true`)

In `app/src/bootstrap.rs`:
- Read `plugins_config` path → read file → parse `PluginsToml` via `toml::from_str`
- If file missing: warn + use `PluginsToml::default()` (empty config, scheduler disabled)
- Construct `PluginRegistry` with `ChromiumWebChecker` (real-plugins) or `NoOpWebChecker` (stub-plugins)

**TDD:** Config tests: `CHROMIUM_PATH` present/absent; `SCHEDULER_ENABLED` default; TOML file read path.

**Acceptance:** `cargo test -p app` green.

---

### C7 · Services — Chapter Check Service
**Crate:** `services` | **Zone:** Impure

New file `services/src/chapter_check.rs`:
```rust
pub struct ChapterCheckService<WC, CCR, NR, WMR> {
    checker: Arc<WC>,
    check_repo: Arc<CCR>,
    notification_repo: Arc<NR>,
    web_meta_repo: Arc<WMR>,
}

impl<...> ChapterCheckService<...> {
    pub async fn check_resource(&self, resource_id: Uuid) -> Result<ChapterCheck, DomainError>;
    pub async fn list_check_history(&self, resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError>;
    pub async fn list_notifications(&self, unread_only: bool) -> Result<Vec<Notification>, DomainError>;
    pub async fn mark_notification_read(&self, id: Uuid) -> Result<(), DomainError>;
}
```

`check_resource` logic:
1. `web_meta_repo.get(resource_id)` → get URL + known `latest_chapter`
2. `checker.check(url, known_chapter)` → `Result<CheckResult, PluginError>` → map to `DomainError::InternalError` on failure
3. `check_repo.create(resource_id, has_new, latest_chapter, error_message)`
4. If `has_new`: `notification_repo.create(resource_id, "New chapter: {latest_chapter}")`; update `web_meta_repo` `last_checked_at` + `last_checked_chapter`
5. Return saved `ChapterCheck`

**TDD:** Unit tests with in-memory mock repos (test doubles for all 4 DI params):
- Normal check, new chapter found → notification created
- No new chapter → no notification
- Checker returns `PluginError::IoError` → error recorded, `DomainError` returned
- `NotFound` for unknown resource_id

**Acceptance:** `cargo test -p services` green.

---

### C8 · App — Background Scheduler
**Crate:** `app` | **Zone:** Impure

New file `app/src/scheduler.rs`:
```rust
pub struct Scheduler {
    check_service: Arc<ChapterCheckService<...>>,
    web_meta_repo: Arc<dyn WebReaderMetaRepository>,
    resource_repo: Arc<dyn ResourceRepository>,
    default_interval: Duration,
    shutdown: CancellationToken,
}

impl Scheduler {
    pub fn start(self) -> JoinHandle<()>;
}
```

Scheduler loop:
1. Query all `WebReader` resources via `resource_repo.list()` + filter `resource_type == WebReader`.
2. For each: get `web_meta.check_interval_secs` (or `default_interval`). Compute if due (compare `last_checked_at`).
3. If due: spawn `check_service.check_resource(id)` — errors logged but do not crash scheduler.
4. Sleep for minimum interval across all resources (floor: 60s).
5. On `CancellationToken` triggered: exit loop cleanly.

Wire scheduler start into `app/src/runtime.rs` when `scheduler_enabled = true`.

**TDD:**
- Unit test for "due" calculation logic (time-injectable via parameter, not `Utc::now()` directly)
- Unit test: scheduler skips resources with no matching site config (no-op checker returns early)
- Integration test: mock service records call, scheduler triggers it within test time window

**Acceptance:** `cargo test -p app` green. Scheduler shuts down cleanly on signal.

---

### C9 · Infrastructure — Firebase FCM Client
**Crate:** `infrastructure` | **Zone:** Impure (behind `firebase` feature)

`FcmClient` struct:
- Config: path or inline JSON of service account from `AppConfig.fcm_service_account`
- Method: `async fn send_notification(&self, resource_id: Uuid, title: &str, body: &str) -> Result<(), FcmError>`
- Uses Firebase Admin SDK HTTP v1 API: `POST https://fcm.googleapis.com/v1/projects/{project_id}/messages:send`
- Auth: generate short-lived OAuth2 Bearer token from service account JSON (use `jsonwebtoken` + `reqwest`)
- Feature gated: `[features] firebase = ["dep:reqwest", "dep:jsonwebtoken", "dep:serde_json"]`

Wiring: `ChapterCheckService` receives an `Option<Arc<FcmClient>>` — if `Some`, call after creating notification.

**TDD:**
- Unit test: FCM request body shape with mock `reqwest` (using `wiremock` or `mockito`)
- Unit test: `FcmClient::new` fails gracefully with invalid JSON (returns error, not panic)
- Test that `ChapterCheckService` calls FCM only when `has_new = true`

**Acceptance:** `cargo test -p infrastructure --features firebase` green. Feature absent: no FCM compilation.

---

### C10 · Adapters — New HTTP Endpoints (Chapter Check + Notifications + Batch Import)
**Crate:** `adapters` | **Zone:** Impure

New file `adapters/src/chapter_check.rs`:
```
POST /api/v1/inventory/web-readers/:id/check
    → ChapterCheckService::check_resource(id)
    → 200 { id, has_new_chapter, latest_chapter, checked_at, error_message }

GET  /api/v1/inventory/web-readers/:id/checks
    → ChapterCheckService::list_check_history(id)
    → 200 [{ ...ChapterCheck }]
```

New file `adapters/src/notifications.rs`:
```
GET  /api/v1/notifications?unread_only=true|false
    → ChapterCheckService::list_notifications(unread_only)
    → 200 [{ id, resource_id, message, created_at, read }]

POST /api/v1/notifications/:id/read
    → ChapterCheckService::mark_notification_read(id)
    → 204

GET  /api/v1/notifications/stream
    → SSE stream using axum::response::Sse
    → Each new notification pushed as: `data: { id, resource_id, message }\n\n`
    → Uses tokio::sync::broadcast::Sender<NotificationEvent> shared via AppState
    → Client receives events until connection dropped
```

New file `adapters/src/batch_import.rs`:
```
POST /api/v1/inventory/ebooks/batch-import
Request: [{ title, notes, author, isbn, publisher, language, file_format, locations: [{device_id, path_or_url, storage_type}] }]
Response: { succeeded: [{ index, resource_id }], failed: [{ index, error }] }
```
- Validate each entry with `use_cases::ebook::validate_new_ebook`
- On validation error: add to `failed` list, continue
- On DB error: add to `failed` list, continue (partial success allowed)
- All inserts are individual (no transaction wrapping the whole batch — partial success is valid)

Update `adapters/src/routes.rs` to register all new routes.  
Update `adapters/src/state.rs`:  
- Add `notification_tx: broadcast::Sender<NotificationEvent>` to `AppState`
- Broadcast from `ChapterCheckService` when a notification is created (use a callback/channel passed in at construction)

**TDD:**
- Handler tests using `axum_test` or `tower::ServiceExt`:
  - Chapter check: 200 with mock service; 404 for unknown resource
  - Notifications list: filtered by unread_only param
  - Batch import: 3-item batch (2 ok + 1 invalid title) → correct succeeded/failed split
  - SSE: connect, trigger broadcast, assert event received

**Acceptance:** `cargo test -p adapters` green. All new routes registered and reachable.

---

### C11 · App — OpenAPI Spec Update
**Crate:** `adapters` + `app` | **Zone:** Impure

- Add `utoipa` annotations to all Phase 2 handlers (request/response schemas)
- Add new schemas: `ChapterCheck`, `Notification`, `BatchImportRequest`, `BatchImportResponse`
- Regenerate Dart client after backend compiles: `openapi-generator-cli generate -i openapi.json -g dart -o frontend/lib/api/`
- Update `frontend/lib/api/.gitignore` patterns if needed

**Acceptance:** `GET /api/v1/system/openapi` returns schema including all Phase 2 endpoints. Dart client compiles.

---

### C12 · Docker — Add Chromium to Alpine Image
**File:** `docker/Dockerfile`, `docker/docker-compose.yml`

- Add to runtime stage:
  ```dockerfile
  RUN apk add --no-cache chromium
  ENV CHROMIUM_PATH=/usr/bin/chromium-browser
  ```
- Set `CHROMIUM_PATH` in `docker-compose.yml` environment section
- If building with `real-plugins` feature: update `cargo build` flags in builder stage

**TDD:** `docker compose build` passes. Smoke test: `docker run ... /usr/bin/chromium-browser --version` exits 0.

**Acceptance:** Container builds and Chromium binary is executable inside runtime image.

---

## Track D — Frontend (Flutter/Dart)

### D1 · Research — Ebook Metadata Extraction Library Survey
**Zone:** Pure research — no code

Evaluate each candidate against: null safety, pub score, platform matrix (Linux x64, macOS arm/x64, Windows x64, Android arm/arm64):

| Format | Candidates to evaluate |
|---|---|
| PDF | `pdf_render`, `syncfusion_flutter_pdf`, `pdfx` |
| EPUB | `epub_parser`, `archive` (manual OPF parse) |
| MOBI | search pub.dev; if none viable → plan FFI |
| AZW3 | search pub.dev; if none viable → plan FFI |

For MOBI/AZW3 fallback (if no viable Dart package):
- Option A: Cross-compile `mobi-rs` Rust crate to `.so`/`.dll`/`.dylib` per platform using `flutter_rust_bridge`
- Option B: Bundle a platform-specific CLI binary in Flutter assets; invoke via `Process.run`
- Option C: Server-side extraction endpoint (fallback only if A+B both impractical)

**Output:** Update `CONTEXT.md` with chosen packages and MOBI/AZW3 path before starting D2.

**Acceptance:** Decision recorded in CONTEXT.md. `flutter pub get` passes with chosen packages added to `pubspec.yaml`.

---

### D2 · Flutter — MetadataExtractor Plugin Abstraction
**Layer:** `lib/plugins/` | **Zone:** Pure

```dart
// lib/plugins/metadata_extractor.dart
abstract interface class MetadataExtractor {
  Set<String> get supportedExtensions;
  Future<Result<ExtractedMeta, AppFailure>> extract(File file);
}

// lib/plugins/extracted_meta.dart
class ExtractedMeta {
  final String? title;
  final String? author;
  final String? isbn;
  final String? publisher;
  final String? language;
  final String? fileFormat;
  const ExtractedMeta({...});
}

// lib/plugins/metadata_extractor_registry.dart
class MetadataExtractorRegistry {
  final List<MetadataExtractor> _extractors;
  MetadataExtractor? forFile(File file);   // matches by extension
  Future<Result<ExtractedMeta, AppFailure>> extract(File file);
}
```

**TDD:** Unit tests:
- Registry returns correct extractor by extension
- Registry returns `AppFailure.unsupported` for unknown extension
- `forFile` case-insensitive extension match

**Acceptance:** `flutter test test/plugins/` green. Zero I/O in abstract layer.

---

### D3 · Flutter — PDF Metadata Extractor
**Layer:** `lib/plugins/` | **Zone:** Impure

`PdfMetadataExtractor implements MetadataExtractor`:
- `supportedExtensions = {'.pdf'}`
- Open file with chosen package; read document info dict
- Map fields: `Title → title`, `Author → author`, `Subject → isbn (if ISBN-shaped)`, `Creator → publisher`
- On error: return `AppFailure.extraction(message)`

Test fixtures: commit a minimal 1-page PDF with known metadata under `test/fixtures/`.

**TDD:**
- Extracts correct title + author from fixture PDF
- Returns `AppFailure.extraction` on corrupted file
- Handles PDF with empty info dict (returns all-null `ExtractedMeta` without error)

**Acceptance:** `flutter test test/plugins/pdf_extractor_test.dart` green on macOS + verified on Linux (CI).

---

### D4 · Flutter — EPUB Metadata Extractor
**Layer:** `lib/plugins/` | **Zone:** Impure

`EpubMetadataExtractor implements MetadataExtractor`:
- `supportedExtensions = {'.epub'}`
- Read EPUB as ZIP using `archive` package (pure Dart, already cross-platform)
- Find `*.opf` file inside ZIP; parse XML
- Extract: `dc:title`, `dc:creator`, `dc:publisher`, `dc:language`, `dc:identifier` (ISBN if URN format)
- On missing OPF: return `AppFailure.extraction("OPF not found")`

Test fixture: commit a minimal valid EPUB (ZIP with `content.opf`) under `test/fixtures/`.

**TDD:**
- Extracts all 5 metadata fields from fixture
- Handles EPUB with partial metadata (missing fields → null, no error)
- Returns failure on non-ZIP file passed as EPUB

**Acceptance:** `flutter test test/plugins/epub_extractor_test.dart` green.

---

### D5 · Flutter — MOBI/AZW3 Metadata Extractor
**Layer:** `lib/plugins/` (or `lib/plugins/ffi/`) | **Zone:** Impure

Implementation path determined in D1 research. All three paths must:
- Return `ExtractedMeta` with at minimum: title + author
- Return `AppFailure.unsupported` gracefully if platform not supported
- Not crash on malformed input

**Path A (native Dart package):**
- Implement `MobiMetadataExtractor` + `Azw3MetadataExtractor` using discovered package
- `supportedExtensions = {'.mobi'}` / `{'.azw3'}`

**Path B (`flutter_rust_bridge` FFI):**
- Create `backend/mobi_extractor` Rust crate with `#[flutter_rust_bridge::frb]` exported function
- Build targets: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, `aarch64-linux-android`, `x86_64-linux-android`
- Dart: generated FFI bindings in `lib/plugins/ffi/`

**Path C (subprocess):**
- Bundle prebuilt CLI binaries per platform in `assets/extractors/`
- `MobiExtractorSubprocess` spawns binary via `Process.run`, parses JSON output
- Platform detection via `Platform.operatingSystem` + `Platform.version`

**TDD:**
- Happy path: extract known MOBI/AZW3 fixture
- Unsupported platform: returns `AppFailure.unsupported` without crash
- Path B only: cross-platform build matrix verified in CI

**Acceptance:** `flutter test test/plugins/mobi_extractor_test.dart` green on all supported platforms (or skipped with clear reason on unsupported ones).

---

### D6 · Flutter — Single Import Flow with Metadata Auto-Fill
**Layer:** `lib/screens/`, `lib/blocs/` | **Zone:** Mixed

Changes to `AddResourceScreen`:
- Add "Pick file" `FilledButton.icon` above form fields
- On tap: `FilePicker.platform.pickFiles(allowedExtensions: ['pdf','epub','mobi','azw3'])` → `File`
- Dispatch `ExtractMetadataRequested(file)` event to `EbookBloc`

New `EbookBloc` events/states:
```dart
class ExtractMetadataRequested extends EbookEvent { final File file; }
class MetadataExtractionState { ExtractedMeta? meta; bool extracting; AppFailure? extractionFailure; }
```

`EbookBloc` handler for `ExtractMetadataRequested`:
1. Emit `extracting = true`
2. `MetadataExtractorRegistry.extract(file)` → `Result<ExtractedMeta, AppFailure>`
3. On success: emit extracted fields into form state (pre-fill title, author, isbn, publisher, language, fileFormat)
4. On failure: emit `extractionFailure`; do NOT block form — user can still fill manually

Form pre-fill: bind text controllers to bloc state; pre-fill only if field currently empty (don't overwrite user edits).

**TDD:**
- Bloc test: `ExtractMetadataRequested` with mock extractor → state has pre-filled fields
- Bloc test: extractor failure → `extractionFailure` set, form fields unchanged
- Widget test: "Pick file" button exists; after extraction, text fields populated

**Acceptance:** `flutter test` green. Manual test: pick a PDF → form fills automatically.

---

### D7 · Flutter — Bulk Import Screen
**Layer:** `lib/screens/`, `lib/blocs/` | **Zone:** Mixed

New `BulkImportScreen`:
- File picker: multi-file OR directory with recursive toggle
- File list with extraction status per file: `pending | extracting | extracted | failed`
- Per-file: show extracted title + error if any; allow manual title override
- "Import All" button: triggers batch upload
- Progress: per-file extraction progress + single POST progress
- Result summary: `X / Y succeeded` with expandable error list

New `BulkImportBloc`:
```dart
events: FilesSelected, ExtractionTriggered, BatchImportRequested
states: BulkImportState { files, extractionStatuses, uploadStatus, result }
```

Implement `HttpBatchOperationRepository` (was a stub in Phase 1):
- `batchImport(BatchImportRequest)` → `POST /api/v1/inventory/ebooks/batch-import`
- Map response to `BatchOperationResponse`

`BatchImportRequest` already modelled in `lib/models/batch_operations.dart` — verify fields align with Phase 2 backend schema; update if needed.

**TDD:**
- Bloc test: `FilesSelected` → files added to state
- Bloc test: `ExtractionTriggered` → each file processed, statuses updated
- Bloc test: `BatchImportRequested` with mock repo returning partial success → result state correct
- Repository test: `HttpBatchOperationRepository.batchImport` sends correct JSON body

**Acceptance:** `flutter test` green. Manual test: pick 3 PDFs → extract → import → see results.

---

### D8 · Flutter — WebView Chapter DOM Detection
**Layer:** `lib/widgets/web_reader_progress_tracker.dart` | **Zone:** Impure

Extend `WebReaderProgressTracker` widget:
- Add `JavaScriptChannel` named `ProgressSignal` to `WebViewController`
- On page load (`onPageFinished`): inject `assets/js/progress_tracker.js`
- JS script must:
  - Override `history.pushState` and `history.replaceState` to emit URL-change events
  - Accept optional `cssSelector` from Flutter (passed via `runJavaScript` before injection)
  - If `cssSelector` provided: attach `MutationObserver` on matched element, emit text on change
  - Emit: `ProgressSignal.postMessage(JSON.stringify({ url: window.location.href, chapterText: text }))`
- On `ProgressSignal` message received: dispatch `TrackWebReaderProgress(url, chapterText)` event

New asset: `assets/js/progress_tracker.js` — register in `pubspec.yaml` assets section.

The `chapterText` sourced from DOM element text (via CSS selector configured per web reader resource). Selector stored in `WebReaderMeta` — add `progress_css_selector: Option<String>` field to `WebReaderMeta` in domain + migration `0010_alter_web_reader_metas_add_progress_selector.sql`.

**TDD:**
- Unit test: JS channel message parsing → correct `TrackWebReaderProgress` event dispatched
- Unit test: null/empty `chapterText` → event not dispatched (no-op)
- Widget test: mock `WebViewController`, verify `addJavaScriptChannel` called with `ProgressSignal`

**Acceptance:** `flutter test` green. Manual test in WebView: navigate page → progress updated in backend.

---

### D9 · Flutter — Notification Handling (SSE + Firebase)
**Layer:** `lib/repositories/`, `lib/blocs/`, `lib/widgets/` | **Zone:** Mixed

New `NotificationRepository`:
```dart
abstract interface class NotificationRepository {
  Stream<NotificationEvent> subscribeToStream();
  Future<Result<List<AppNotification>, AppFailure>> listNotifications({bool unreadOnly = true});
  Future<Result<void, AppFailure>> markRead(String id);
}

class HttpNotificationRepository implements NotificationRepository {
  // SSE: connect to GET /api/v1/notifications/stream using http package chunked response
  // Parse "data: {...}\n\n" SSE lines
}
```

Firebase setup:
- Add `firebase_messaging: ^14.x` to `pubspec.yaml`
- `FirebaseMessaging.instance.requestPermission()` on app start
- `FirebaseMessaging.onMessage` (foreground): dispatch to `NotificationBloc`
- `FirebaseMessaging.onBackgroundMessage` handler (background)
- FCM token registration: `POST /api/v1/devices/fcm-token` (new endpoint — add to backend in C10 extension)

New `NotificationBloc`:
```dart
events: SubscribeToStream, NotificationReceived, MarkNotificationRead
states: NotificationState { List<AppNotification> notifications, bool hasUnread }
```

In-app notification banner: `OverlayEntry` or `ScaffoldMessenger.showSnackBar` on `NotificationReceived` state.

Notification list screen (optional sub-screen in settings/drawer): list + mark read + clear all.

**TDD:**
- Repository test: mock HTTP chunked response → `Stream<NotificationEvent>` emits correctly
- Bloc test: `NotificationReceived` → `hasUnread = true`; `MarkNotificationRead` → item removed from unread
- Bloc test: stream subscription active on bloc creation

**Acceptance:** `flutter test` green. Manual test: trigger backend chapter check → in-app banner appears.

---

### D10 · Flutter — Chapter Check History UI
**Layer:** `lib/screens/web_reader_detail_screen.dart`, `lib/blocs/` | **Zone:** Mixed

Extend `WebReaderDetailScreen`:
- Add "Chapter Checks" expandable section below existing content
- `GET /api/v1/inventory/web-readers/:id/checks` on screen load → display history list
- Each row: timestamp, `has_new_chapter` indicator, `latest_chapter`, error (if any)
- "Check Now" `IconButton` in app bar or detail body → `POST /api/v1/inventory/web-readers/:id/check` → refresh history + show result snackbar

New `WebReaderBloc` events/states:
```dart
class TriggerChapterCheck extends WebReaderEvent { final String resourceId; }
class LoadCheckHistory extends WebReaderEvent { final String resourceId; }
class CheckHistoryLoaded extends WebReaderState { final List<ChapterCheck> history; }
class ChapterCheckTriggered extends WebReaderState { final ChapterCheck result; }
```

Add `ChapterCheck` model + `CheckResult` to `lib/models/`.  
Add methods to `WebReaderRepository`:
```dart
Future<Result<ChapterCheck, AppFailure>> triggerCheck(String resourceId);
Future<Result<List<ChapterCheck>, AppFailure>> listCheckHistory(String resourceId);
```

**TDD:**
- Bloc test: `TriggerChapterCheck` → calls repo, emits `ChapterCheckTriggered`
- Bloc test: `LoadCheckHistory` → calls repo, emits `CheckHistoryLoaded`
- Repository test: HTTP calls map to correct endpoints

**Acceptance:** `flutter test` green. Check history visible in detail screen.

---

## Track E — Cross-Cutting

### E1 · CONTEXT.md Update
- Record all Phase 2 decisions (library choices from D1, FCM project setup, CHROMIUM_PATH, scheduler config)
- Add new API endpoints to architecture section
- Update "Phases Overview" Phase 2 status on completion

**Acceptance:** CONTEXT.md accurate and up-to-date before Phase 3 begins.

---

## Task Dependency Order

```
C1 → C2 → C3
C4 → C5 → C7 → C8
C6 → C8
C9 → C10
C10 (batch) standalone
C11 → C7 (wire FCM into service)
C12 standalone (Docker)
C1 → C7
C3 → C7
C10 → C11 (adapters wire notification broadcast)
D1 → D2 → D3, D4, D5
D2 → D6 → D7
D6, D7 → E1
D8 standalone (extends Phase 1 WebView stub)
D9 → D10
Backend C10 (SSE) must be complete before D9
Backend C10 (chapter check endpoints) must be complete before D10
Backend C10 (batch import) must be complete before D7
Backend C11 (OpenAPI update) must be complete before Flutter api/ regeneration
```

---

## TDD Checkpoint Rule (from AGENTS.md)

After writing each test set (Red phase): **STOP. Manual review required before proceeding to Green.**  
Resume only after explicit prompt confirmation.

---

## Acceptance Summary

| Track | Gate |
|---|---|
| C1–C3 | `cargo test -p domain` + `cargo test -p infrastructure` green |
| C4–C6 | `cargo test -p plugins` green (stub + real-plugins features) |
| C7–C9 | `cargo test -p services` + `cargo test -p app` green |
| C10 | `cargo test -p adapters` green; all new routes registered |
| C11 | `cargo test -p infrastructure --features firebase` green |
| C12 | Docker build passes; Chromium binary present in image |
| D1 | Decision in CONTEXT.md; `flutter pub get` clean |
| D2–D5 | `flutter test test/plugins/` green |
| D6–D7 | `flutter test` green; manual file import verified |
| D8 | `flutter test` green; manual WebView progress verified |
| D9–D10 | `flutter test` green; manual notification + check history verified |
| E1 | CONTEXT.md updated with all Phase 2 decisions |
