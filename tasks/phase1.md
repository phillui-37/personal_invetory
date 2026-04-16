# Phase 1 Tasks — Personal Inventory MVP

**Spec:** `docs/superpowers/specs/2026-04-15-phase1-mvp-design.md`  
**Approach:** Backend-first (full backend → Flutter shell)  
**Methodology:** TDD — Red → Green → Refactor on every task  
**Resource types in scope:** Ebook, WebReader only

## Critical Alignment Fixes (from TODO.md + AGENTS.md)

This plan is corrected to explicitly cover required items that were previously implicit or deferred:
- Device identity lifecycle (unique IDs, ID reuse and delink semantics)
- PostgreSQL ↔ SQLite round-trip portability validation
- Fuzzy search path (phase-1 compatible baseline + phase-next extension hook)
- Web progress tracking (URL + DOM signal path)
- Batch operations (import, metadata update/copy) as planned implementation track

Also aligned with `AGENTS.md`:
- No panic-driven control flow for recoverable startup/config errors
- Mandatory per-task TDD gate: write failing test first, verify red, then implementation

---

## Architecture Principles

### Pure / Impure Separation
| Zone | Rule |
|---|---|
| **Pure** | No I/O, no async, no exceptions. Returns typed `Result`/`Option` for expected failures (e.g. validation). Always deterministic. |
| **Impure** | All I/O lives here (DB, HTTP, filesystem, user interaction). Catches all exceptions at the boundary and converts to typed errors. Nothing above this layer uses try/catch or untyped panics. |

### Backend Layer Order (dependency direction)
```
adapters → services → use_cases → domain ← infrastructure
                                  ↑
                               plugins
```

### Flutter Layer Order
```
screens/widgets (impure UI)
    ↓ events
  blocs (pure logic)
    ↓ calls
  repositories (impure I/O boundary)
    ↓ HTTP
  api/ (generated client, impure)
```

---

## Track A — Backend (Rust)

### A0 · Plan-Requirement Lock & Scope Guard

Create a machine-checkable requirements matrix (`tasks/phase1-requirements-matrix.json`) mapping each requirement in `TODO.md` to:
- `phase`: `phase1` or `deferred`
- `task_ids`: concrete task IDs in this plan
- `status`: `planned`

Include explicit entries for:
- Device ID uniqueness + reuse/delink behavior
- DB portability
- Fuzzy search
- Web URL/DOM progress signals
- Batch operations

**Acceptance:** Every requirement line from `TODO.md` has exactly one mapping record; no unmapped requirement remains.

---

### A1 · Rust Workspace Scaffold

Set up `backend/Cargo.toml` as a Cargo workspace with **seven** member crates:
`domain`, `use_cases`, `plugins`, `services`, `adapters`, `infrastructure`, `app`

Inter-crate dependency rules (strictly enforced — no cycles):
- `use_cases` → `domain`
- `plugins` → `domain`
- `services` → `domain`, `use_cases`, `plugins`
- `adapters` → `domain`, `services`, `plugins`
- `infrastructure` → `domain`, `plugins`
- `app` → `domain`, `use_cases`, `plugins`, `services`, `adapters`, `infrastructure`

Each crate gets a minimal `Cargo.toml`. No crate may import from a crate above itself in the dependency order — enforced by the compiler.

**Acceptance:** `cargo check --workspace` passes with zero errors and zero cycles.

---

### A2 · Domain — Entities & Value Objects
**Crate:** `domain` | **Zone:** Pure

Define all core types. No `async`, no I/O, no external deps beyond `uuid`, `chrono`, `serde`.

Types to define:
- `ResourceType` enum: `Ebook`, `WebReader`
- `StorageType` enum: `LocalFs`, `Nas`, `Platform`, `Portable`
- `Resource` struct: `id: Uuid`, `title: String`, `notes: Option<String>`, `resource_type: ResourceType`, `created_at/updated_at: DateTime<Utc>`
- `EbookMeta`: `resource_id: Uuid`, `author/isbn/publisher/language/file_format: Option<String>`
- `WebReaderMeta`: `resource_id: Uuid`, `url: String`, `site_name: Option<String>`, `last_checked_chapter: Option<String>`
- `ResourceLocation`: `id: Uuid`, `resource_id: Uuid`, `device_id: String`, `path_or_url: String`, `storage_type: StorageType`
- Input structs: `NewResource`, `UpdateResource`, `NewEbookMeta`, `NewWebReaderMeta`, `NewResourceLocation`

All types derive `Debug`, `Clone`, `Serialize`, `Deserialize`.

**TDD:** Tests for construction, serde round-trip, display formatting.

**Acceptance:** `cargo test -p domain` green. Zero I/O in crate.

---

### A3 · Domain — Repository Traits & Domain Error
**Crate:** `domain` | **Zone:** Pure (trait definitions only — no implementations)

`DomainError` enum:
```rust
pub enum DomainError {
    NotFound(String),
    ValidationError(String),
    Conflict(String),
    InternalError(String),
}
```

Repository traits (all `#[async_trait]`, `Send + Sync`):
```
ResourceRepository:     list, search(query), get_by_id, create, update, delete
EbookMetaRepository:    get(resource_id), upsert
WebReaderMetaRepository: get(resource_id), upsert
LocationRepository:     list(resource_id), add, remove
```
All methods return `Result<T, DomainError>`.

**TDD:** In-memory mock implementations of each trait. Unit tests verify trait contracts (CRUD, error cases).

**Acceptance:** `cargo test -p domain` green. No `async` implementation code in this crate — traits only.

---

### A4 · Use Cases — Pure Use Case Functions
**Crate:** `use_cases` | **Zone:** Pure

One module per resource type: `ebook.rs`, `web_reader.rs`, `location.rs`.

Each module contains free functions (not methods) that:
- Accept input DTOs + domain types
- Apply validation and business rules
- Return `Result<DomainOutputType, ValidationError>` — no `async`, no I/O

`ValidationError` is a `use_cases`-local type (not `DomainError`) — it carries field-level validation messages.

Examples:
```rust
// ebook.rs
pub fn validate_new_ebook(input: &NewEbookInput) -> Result<(NewResource, NewEbookMeta), ValidationError>
pub fn validate_update_ebook(existing: &Resource, input: &UpdateEbookInput) -> Result<(UpdateResource, NewEbookMeta), ValidationError>

// web_reader.rs
pub fn validate_new_web_reader(input: &NewWebReaderInput) -> Result<(NewResource, NewWebReaderMeta), ValidationError>

// location.rs
pub fn validate_new_location(input: &NewLocationInput) -> Result<NewResourceLocation, ValidationError>
```

Validations enforced:
- Title: non-empty, max 500 chars
- WebReader URL: non-empty, must parse as a valid URL
- StorageType: must be a known variant
- file_format (ebook): if set, must be one of `pdf`, `epub`, `mobi`, `azw3`

**TDD:** Unit tests per function covering valid inputs, each validation rule failure, and boundary values.

**Acceptance:** `cargo test -p use_cases` green. Zero `async`, zero I/O, zero `DomainError` (only `ValidationError`).

---

### A5 · Plugins — Trait Interfaces & No-Op Stubs
**Crate:** `plugins` | **Zone:** Pure (trait definitions + stubs)

Types:
```rust
pub struct ExtractedMeta { pub title: Option<String>, pub author: Option<String>, pub extra: HashMap<String, String> }
pub struct CheckResult { pub has_new: bool, pub latest_chapter: Option<String> }
pub enum PluginError { IoError(String), ParseError(String), UnsupportedInput }
```

Traits:
```rust
pub trait MetadataExtractor: Send + Sync {
    fn resource_type(&self) -> ResourceType;
    fn extract(&self, input: &str) -> Result<ExtractedMeta, PluginError>;
}
pub trait WebChecker: Send + Sync {
    fn check(&self, url: &str) -> Result<CheckResult, PluginError>;
}
```

No-op stubs behind `#[cfg(feature = "stub-plugins")]` (default feature in dev):
- `NoOpMetadataExtractor` — returns `Ok(ExtractedMeta::empty())`
- `NoOpWebChecker` — returns `Ok(CheckResult { has_new: false, latest_chapter: None })`

`PluginRegistry { extractors: Vec<Box<dyn MetadataExtractor>>, checkers: Vec<Box<dyn WebChecker>> }` with `from_config(config: &PluginsConfig)` constructor.

**TDD:** Unit tests: stubs return correct no-ops, registry registers correct stubs from config.

**Acceptance:** `cargo test -p plugins` passes with and without `--features stub-plugins`.

---

### A6 · Infrastructure — DB Migrations
**Crate:** `infrastructure` | **Zone:** Impure

SQL migrations in `infrastructure/migrations/`. **Must be compatible with PostgreSQL and SQLite:**
- UUIDs → `TEXT`
- Timestamps → `TEXT` (ISO 8601)
- No `JSONB`, `ARRAY`, `SERIAL` — use `TEXT PRIMARY KEY` for UUID PKs
- Enum values → `TEXT`

Migration files:
1. `0001_create_resources.sql`
2. `0002_create_ebook_metas.sql` (FK to resources, ON DELETE CASCADE)
3. `0003_create_web_reader_metas.sql` (FK to resources, ON DELETE CASCADE)
4. `0004_create_resource_locations.sql` (FK to resources, ON DELETE CASCADE)
5. `0005_create_devices.sql` (device registry with unique active device ID semantics)

Indices:
- `resources`: `resource_type`, `title`
- `ebook_metas`: `author`
- `web_reader_metas`: `url`
- `resource_locations`: `resource_id`

**TDD:** Migration test runs all migrations against in-memory SQLite, verifies all tables and columns exist.

**Acceptance:** Migration test green. Manual verification against local PostgreSQL.

---

### A6b · Infrastructure — Device Identity Semantics
**Crate:** `infrastructure` + `services` | **Zone:** Impure

Implement device identity behavior required by `TODO.md`:
- `devices` table tracks active device IDs
- Reusing an existing `device_id` on a new device registration delinks old device association and rebinds to the new device record
- Portable storage locations still allow free-text device IDs in location input

Service-level rules:
- `register_device` enforces "one active owner per device_id"
- `rebind_device_id` keeps historical linkage auditable via timestamps/status

**TDD:** Integration tests for first registration, re-registration (reuse), and old-device delink behavior.

**Acceptance:** Tests confirm deterministic ID reuse behavior with no duplicate active owners.

---

### A7 · Infrastructure — Repository Adapters & Adapter Factory
**Crate:** `infrastructure` | **Zone:** Impure

Implements all four repository traits from `domain` for both PostgreSQL and SQLite via `sqlx`.

Module structure:
```
infrastructure/src/
├── postgres/
│   ├── resource.rs
│   ├── ebook_meta.rs
│   ├── web_reader_meta.rs
│   └── location.rs
├── sqlite/
│   ├── resource.rs
│   ├── ebook_meta.rs
│   ├── web_reader_meta.rs
│   └── location.rs
└── factory.rs
```

LIKE search:
- PostgreSQL: `ILIKE '%q%'` on `title`, `author`, `url`, `site_name`
- SQLite: `LOWER(col) LIKE LOWER('%q%')`

`AdapterFactory::from_url(database_url: &str) -> Result<AdapterBundle, DomainError>` routes by URL prefix. Unknown prefix returns typed startup/config error (no panic).

**All sqlx errors are caught here and mapped to `DomainError::InternalError` — they never escape this crate.**

**TDD:** Integration tests against SQLite in-memory for all adapter methods. PostgreSQL tests `#[ignore]`'d (run in CI).

**Acceptance:** `cargo test -p infrastructure` green (SQLite). PostgreSQL suite green in CI.

---

### A7b · Infrastructure — Cross-DB Portability Verification
**Crate:** `infrastructure` | **Zone:** Impure

Add canonical export/import pipeline for portability checks:
- Export canonical JSON snapshot from SQLite
- Import into PostgreSQL
- Re-export PostgreSQL snapshot
- Compare normalized snapshots for equality

Normalization rules:
- Stable ordering by primary keys
- Canonical datetime formatting
- No backend-specific field drift

**TDD:** Portability integration test fixture with mixed Ebook/WebReader/resource_locations records.

**Acceptance:** SQLite → PostgreSQL → SQLite round-trip yields data-equivalent canonical snapshots.

---

### A7c · Infrastructure — Fuzzy Search Extension Hook
**Crate:** `domain` + `infrastructure` + `services`

Phase-1 behavior remains LIKE-based by default, but add extension seam:
- `SearchStrategy` trait with `Like` and `Fuzzy` implementations
- Config key selects strategy per backend
- SQLite supports FTS5 path; PostgreSQL supports trigram/similarity path

**TDD:** Contract tests ensuring both strategies return deterministic ranked results for the same fixture set.

**Acceptance:** Default strategy remains LIKE; fuzzy strategy can be switched on without service-layer changes.

---

### A8 · Services — Async Executors
**Crate:** `services` | **Zone:** Impure boundary (pure use cases + impure repo calls)

One `Service` struct per resource type, each holding `Arc` references to the relevant repository traits:

```rust
pub struct EbookService { resource_repo: Arc<dyn ResourceRepository>, ebook_meta_repo: Arc<dyn EbookMetaRepository>, location_repo: Arc<dyn LocationRepository> }
pub struct WebReaderService { ... }
```

Each method follows the pattern:
1. Call pure use case function → map `ValidationError` to `DomainError::ValidationError`
2. Call repository trait method(s) (async) → IO errors already typed as `DomainError`
3. Return `Result<T, DomainError>`

Services do **not** use try/catch — repository errors are already typed. Services are the **only** layer that combine pure use case results with async I/O calls.

**TDD:** Integration tests with mock repository implementations (from domain's test helpers). Test happy paths and each error variant.

**Acceptance:** `cargo test -p services` green.

---

### A9 · Adapters — HTTP Setup & Auth Middleware
**Crate:** `adapters` | **Zone:** Impure

`AppState` struct holding `Arc<EbookService>`, `Arc<WebReaderService>`, `Arc<PluginRegistry>`, `api_key: String`, `openapi_json: String`.

Auth middleware (`tower::Layer`):
- Extracts `Authorization: Bearer <token>` header
- Compares against `api_key` in `AppState`
- Returns `401` with `{ "error": "Invalid or missing API key" }` on mismatch
- `/api/v1/system/health` and `/api/v1/system/openapi` bypass auth

`ApiError` type: maps `DomainError` → HTTP status + JSON:
- `NotFound` → 404
- `ValidationError` → 422
- `Conflict` → 409
- `InternalError` → 500

**No other layer outside `adapters` knows about HTTP status codes.**

**TDD:** Unit tests: auth middleware with valid/missing/wrong key. `ApiError` mapping for each `DomainError` variant.

**Acceptance:** `cargo test -p adapters` auth + error tests green.

---

### A10 · Adapters — Ebook HTTP Handlers
**Crate:** `adapters` | **Zone:** Impure

All request/response structs derive `utoipa::ToSchema`. Handlers are pure wrappers: extract request → call `EbookService` method → map to response.

| Handler fn | Endpoint |
|---|---|
| `list_ebooks` | `GET /api/v1/inventory/ebooks/list` |
| `search_ebooks` | `GET /api/v1/inventory/ebooks/search?q=` |
| `add_ebook` | `POST /api/v1/inventory/ebooks/add` |
| `ebook_detail` | `GET /api/v1/inventory/ebooks/:id/detail` |
| `update_ebook` | `PUT /api/v1/inventory/ebooks/:id/update` |
| `delete_ebook` | `DELETE /api/v1/inventory/ebooks/:id/delete` |
| `add_ebook_location` | `POST /api/v1/inventory/ebooks/:id/locations/add` |
| `remove_ebook_location` | `DELETE /api/v1/inventory/ebooks/:id/locations/:loc_id/remove` |

**TDD:** Integration tests via `axum::test` with mock `EbookService`. Test happy path + all error paths (400/404/422/500).

**Acceptance:** `cargo test -p adapters` ebook handler tests green.

---

### A11 · Adapters — Web-Reader HTTP Handlers
**Crate:** `adapters` | **Zone:** Impure

Mirror of A10 for web-reader type.

| Handler fn | Endpoint |
|---|---|
| `list_web_readers` | `GET /api/v1/inventory/web-readers/list` |
| `search_web_readers` | `GET /api/v1/inventory/web-readers/search?q=` |
| `add_web_reader` | `POST /api/v1/inventory/web-readers/add` |
| `web_reader_detail` | `GET /api/v1/inventory/web-readers/:id/detail` |
| `update_web_reader` | `PUT /api/v1/inventory/web-readers/:id/update` |
| `delete_web_reader` | `DELETE /api/v1/inventory/web-readers/:id/delete` |
| `add_web_reader_location` | `POST /api/v1/inventory/web-readers/:id/locations/add` |
| `remove_web_reader_location` | `DELETE /api/v1/inventory/web-readers/:id/locations/:loc_id/remove` |

**TDD:** Same pattern as A10.

**Acceptance:** `cargo test -p adapters` web-reader handler tests green.

---

### A12 · Adapters — System Handlers & OpenAPI
**Crate:** `adapters` | **Zone:** Impure

- `GET /api/v1/system/health` → `{ "status": "ok" }` (200, no auth)
- `GET /api/v1/system/openapi` → serves pre-built `openapi.json` from `AppState` (no auth)

Add `#[utoipa::path(...)]` annotations to all handlers (A9–A12). Define `ApiDoc` via `utoipa::OpenApi` derive, registering all schemas and paths. Generate spec string at startup, store in `AppState`.

**TDD:** Health returns 200 without auth token. OpenAPI endpoint returns valid parseable JSON with non-empty `paths`.

**Acceptance:** `cargo test -p adapters` system tests green.

---

### A13 · App Binary — Config, API Key Bootstrap & Wiring
**Crate:** `app` | **Zone:** Impure

Config loaded from environment via `dotenvy`:
```
DATABASE_URL      required
API_KEY           optional — bootstrap if absent
HOST              default: 0.0.0.0
PORT              default: 8080
PLUGINS_CONFIG    default: plugins.toml (path)
```

API key bootstrap (runs before anything else):
1. Read `API_KEY` from env
2. If present → use it
3. If absent → generate UUIDv4 → attempt write to `.env` (best-effort, log on failure) → print `⚠️ WARNING: No API_KEY found. Generated: <key>. Add API_KEY=<key> to your .env and restart.` → `std::process::exit(1)`

Startup sequence:
1. Config load
2. API key bootstrap
3. `sqlx::migrate!()` — run pending migrations
4. `AdapterFactory::from_url()` → `AdapterBundle`
5. `PluginRegistry::from_config()` → registry
6. Construct `EbookService`, `WebReaderService`
7. Generate OpenAPI spec string
8. Build `AppState`, build axum router, bind and serve

**TDD:** Integration test: start server against test SQLite DB, `/health` returns 200, unauthenticated request returns 401.

**Acceptance:** `cargo run -p app` starts successfully. Health check responds.

---

### A14 · Docker — Dockerfile & docker-compose
**Zone:** Impure / Infrastructure

`docker/Dockerfile` (multi-stage):
```dockerfile
# Stage 1: builder
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl \
    && rm -rf target/x86_64-unknown-linux-musl/release/{build,deps,incremental} \
    && rm -rf /root/.cargo/registry /root/.cargo/git

# Stage 2: runtime
FROM alpine:latest AS runtime
RUN apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*
WORKDIR /app
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/app .
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser
EXPOSE 8080
ENTRYPOINT ["./app"]
```

`docker/docker-compose.yml`:
- `app`: build context `..`, `env_file: ../.env`, depends_on `postgres`, healthcheck on `/api/v1/system/health` every 30s
- `postgres`: `postgres:16-alpine`, named volume, `POSTGRES_*` from `.env`

`.env.example` documents all required and optional vars.

**Acceptance:** `docker compose up --build` starts both services. `curl localhost:8080/api/v1/system/health` returns `{"status":"ok"}`.

---

## Track B — Frontend (Flutter / BLoC)

_Starts after A14 is complete and the backend OpenAPI spec is available._

### B1 · Flutter Project Scaffold

`flutter create frontend/` with minimum Dart SDK set to 3.0 (for sealed classes).

`pubspec.yaml` dependencies:
- `flutter_bloc` — BLoC state management
- `equatable` — value equality for BLoC states/events
- `fpdart` — NOT used (we use Dart 3 sealed classes instead)
- `http` — HTTP client (or rely on generated client)
- `flutter_dotenv` — for local dev `.env` (CI uses `--dart-define`)

Folder structure:
```
frontend/lib/
├── api/            # generated OpenAPI client (gitignored)
├── config/         # AppConfig (dart-define: BASE_URL, API_KEY)
├── models/         # sealed domain types + sealed Result<T, Failure>
├── repositories/   # abstract interfaces + HTTP implementations (impure)
├── blocs/          # BLoC classes per feature (pure logic)
│   ├── ebook/
│   └── web_reader/
├── screens/        # Flutter screens (impure, dispatch BLoC events)
└── widgets/        # shared reusable widgets
```

**Acceptance:** `flutter run` launches without errors. Folder structure in place.

---

### B2 · Flutter — Sealed Result Type & Domain Models
**Zone:** Pure

Define `Result<T, F>` as a Dart 3 sealed class:
```dart
sealed class Result<T, F> {}
final class Success<T, F> extends Result<T, F> { final T value; }
final class Failure<T, F> extends Result<T, F> { final F failure; }
```

Define `Failure` sealed class hierarchy:
```dart
sealed class AppFailure {}
final class NetworkFailure extends AppFailure { final String message; }
final class NotFoundFailure extends AppFailure { final String id; }
final class ValidationFailure extends AppFailure { final String field; final String message; }
final class ServerFailure extends AppFailure { final int statusCode; }
```

Define Dart model classes mirroring the backend domain (pure data classes with `==` + `hashCode` via `Equatable` or manual):
- `Resource`, `EbookMeta`, `WebReaderMeta`, `ResourceLocation`, `ResourceType`, `StorageType`
- Composite view models: `EbookDetail`, `WebReaderDetail` (resource + meta + locations combined)

**TDD:** Unit tests: `Result` fold exhaustiveness, model equality.

**Acceptance:** `flutter test` model + Result tests green.

---

### B3 · Flutter — API Client Generation & Repository Interfaces
**Zone:** Generated (impure) + interfaces (pure)

Setup `openapi-generator` (Dart target). Add `scripts/gen-api-client.sh`:
```sh
#!/bin/sh
openapi-generator-cli generate \
  -i http://localhost:8080/api/v1/system/openapi \
  -g dart \
  -o frontend/lib/api/
```

`frontend/lib/api/` added to `.gitignore`.

`AppConfig` class reads `BASE_URL` and `API_KEY` from `--dart-define`. Injects `Authorization: Bearer <key>` header into generated client via base options.

Define repository abstract interfaces (pure — no `async` in interface definition beyond `Future<Result<T, AppFailure>>`):
```dart
abstract class EbookRepository {
  Future<Result<List<EbookDetail>, AppFailure>> listEbooks();
  Future<Result<List<EbookDetail>, AppFailure>> searchEbooks(String query);
  Future<Result<EbookDetail, AppFailure>> getEbook(String id);
  Future<Result<EbookDetail, AppFailure>> addEbook(NewEbookInput input);
  Future<Result<EbookDetail, AppFailure>> updateEbook(String id, UpdateEbookInput input);
  Future<Result<void, AppFailure>> deleteEbook(String id);
  Future<Result<ResourceLocation, AppFailure>> addLocation(String resourceId, NewLocationInput input);
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId);
}
// Same pattern for WebReaderRepository
```

Implement `HttpEbookRepository` and `HttpWebReaderRepository`:
- Call generated API client methods
- Wrap in `try/catch` — **this is the only place in Flutter that catches exceptions**
- Map to `Result<T, AppFailure>`

**TDD:** Unit tests for `HttpEbookRepository` using a mock HTTP client. Test exception-to-`Result` mapping.

**Acceptance:** Generated client compiles. `flutter test` repository tests green.

---

### B4 · Flutter — Ebook BLoC
**Zone:** Pure logic

`EbookBloc` with events and states:

```dart
// Events
sealed class EbookEvent {}
final class LoadEbooks extends EbookEvent {}
final class SearchEbooks extends EbookEvent { final String query; }
final class AddEbook extends EbookEvent { final NewEbookInput input; }
final class UpdateEbook extends EbookEvent { final String id; final UpdateEbookInput input; }
final class DeleteEbook extends EbookEvent { final String id; }
final class LoadEbookDetail extends EbookEvent { final String id; }
final class AddEbookLocation extends EbookEvent { ... }
final class RemoveEbookLocation extends EbookEvent { ... }

// States
sealed class EbookState {}
final class EbookInitial extends EbookState {}
final class EbookLoading extends EbookState {}
final class EbookListLoaded extends EbookState { final List<EbookDetail> ebooks; }
final class EbookDetailLoaded extends EbookState { final EbookDetail ebook; }
final class EbookOperationSuccess extends EbookState {}
final class EbookError extends EbookState { final AppFailure failure; }
```

`EbookBloc` receives `EbookRepository` via constructor injection. For each event:
1. Emit `EbookLoading`
2. `await repository.method()` → `Result`
3. Exhaustive switch on `Result` → emit typed state
4. **No try/catch anywhere in BLoC**

**TDD:** Unit tests using `bloc_test` package. Mock `EbookRepository`. Test every event → expected state transitions including failure paths.

**Acceptance:** `flutter test` BLoC tests green. No try/catch in BLoC code (verified by code review).

---

### B5 · Flutter — Web-Reader BLoC
**Zone:** Pure logic

Mirror of B4 for WebReader type. Events/states follow identical pattern with `WebReader` prefix.

**Acceptance:** `flutter test` BLoC tests green.

---

### B6 · Flutter — Resource List Screen
**Zone:** Impure (UI)

`ResourceListScreen` with two tabs: **Ebooks** and **Web Readers**.

Each tab:
- `BlocBuilder<EbookBloc, EbookState>` (or `WebReaderBloc`)
- `EbookInitial` → trigger `LoadEbooks` event on mount
- `EbookLoading` → `CircularProgressIndicator`
- `EbookListLoaded` → `ListView` of cards (title + storage type badge)
- `EbookError` → error message with retry button

Pull-to-refresh → dispatch `LoadEbooks`/`LoadWebReaders`.  
Tap card → navigate to `ResourceDetailScreen`.  
FAB → navigate to `AddResourceScreen`.

**TDD:** Widget tests with mocked BLoC: loading state, loaded state with items, error state, tap navigation.

**Acceptance:** `flutter test` list screen widget tests green.

---

### B7 · Flutter — Add Resource Screen
**Zone:** Impure (UI)

`AddResourceScreen`:
- `DropdownButton` for resource type (Ebook / Web Reader) — controls which fields show
- Common fields: Title (required), Notes
- Ebook-specific: Author, ISBN, Publisher, Language, File Format (dropdown)
- WebReader-specific: URL (required), Site Name
- Location section: Device ID, Path/URL, Storage Type (dropdown)
- Submit → dispatch `AddEbook` or `AddWebReader` event
- `BlocListener`: on `EbookOperationSuccess` → pop + snackbar; on `EbookError` → snackbar

**No validation logic in the screen** — validation is in use cases (backend) and will surface as `ValidationFailure` in BLoC state, displayed as field-level error messages.

**TDD:** Widget tests for type selector switching, required field error display, submit dispatch.

**Acceptance:** `flutter test` add screen widget tests green.

---

### B8 · Flutter — Search Screen
**Zone:** Impure (UI)

`SearchScreen`:
- `TextField` — debounced 400ms, dispatches `SearchEbooks`/`SearchWebReaders`
- Results list: interleaved ebook + web-reader results with type badge
- `EbookInitial` / empty query → "Type to search…"
- `EbookLoading` → `LinearProgressIndicator`
- `EbookListLoaded` (empty) → "No results found"
- Tap → `ResourceDetailScreen`

Uses both `EbookBloc` and `WebReaderBloc` simultaneously (separate `BlocProvider`s).

**TDD:** Widget tests: debounce fires correct event, empty state, results render, navigation.

**Acceptance:** `flutter test` search screen widget tests green.

---

### B9 · Flutter — Resource Detail Screen
**Zone:** Impure (UI)

`ResourceDetailScreen`:
- `BlocBuilder` renders all fields read-only
- Locations section: list with storage type chip per row + delete icon (confirmation dialog → `RemoveEbookLocation` event)
- "Add Location" button → bottom sheet form → `AddEbookLocation` event
- AppBar: **Edit** → `AddResourceScreen` in edit mode (pre-filled); **Delete** → confirmation dialog → `DeleteEbook` → pop to list on `EbookOperationSuccess`

**TDD:** Widget tests: fields display, location list renders, delete confirm dialog, edit navigation.

**Acceptance:** `flutter test` detail screen widget tests green.

---

### B10 · Flutter — Integration Test: Add-Resource Flow
**Zone:** Impure (E2E)

Integration test (`integration_test/`) against a running backend in SQLite mode:

1. App launches → Resource List shows empty tabs
2. Tap FAB → Add Resource screen
3. Select "Ebook", fill title + author, add one location
4. Submit → verify ebook appears in Ebooks tab
5. Tap item → Detail screen shows correct fields and location
6. Tap Delete → confirm → verify item removed from list

**Acceptance:** `flutter test integration_test/` passes against running backend.

---

### B11 · Flutter + Backend — WebView Progress Tracking (URL + DOM)

Implement URL/DOM-only progress tracking path for web-reader resources:
- Embed WebView in web-reader detail flow
- Inject JS bridge that captures URL and chapter/progress signal from DOM selectors
- Send progress updates to backend endpoint
- Backend persists last known chapter/progress marker for that resource

**TDD:** Widget/integration tests for progress signal emission and backend update invocation; backend handler tests for update validation and persistence.

**Acceptance:** Browsing in WebView updates stored progress using only URL + DOM-derived data.

---

### B12 · Flutter + Backend — Batch Operations

Deliver batch operations required by `TODO.md`:
- Batch import resources (multi-file / directory, optional recursive)
- Batch metadata update
- Batch metadata copy

Backend:
- Bulk endpoints with itemized success/failure response
- Partial-failure handling with stable per-item error payloads

Frontend:
- Batch operation screens/dialogs
- Progress + summary reporting

**TDD:** Backend bulk endpoint tests and frontend widget/integration tests for mixed success/failure batches.

**Acceptance:** User can execute all three batch operations with clear per-item results.

---

## Task Dependency Graph

```
A0 (requirements matrix)
  └─ A1 (workspace)
  └─ A2 (domain entities)
       └─ A3 (domain traits + errors)
            ├─ A4 (use_cases)            pure ─────────────────────┐
            ├─ A5 (plugins)              pure                       │
            ├─ A6 (migrations)                                      │
            ├─ A6b (device identity)                                │
            ├─ A7 (DB adapters)          impure                     │
            ├─ A7b (portability checks)                             │
            └─ A7c (fuzzy extension seam)                           │
                 └─ A8 (services)        impure boundary ← A4, A5   │
                      └─ A9 (HTTP setup) impure                     │
                           ├─ A10 (ebook handlers)                  │
                           ├─ A11 (web-reader handlers)             │
                          └─ A12 (system + OpenAPI)                │
                               └─ A13 (app binary)                 │
                                    └─ A14 (Docker)                │
                                                                   │
A14 complete (OpenAPI available)                                   │
 └─ B1 (Flutter scaffold)                                          │
      └─ B2 (models + Result type)      pure ────────────────────┘ │
           └─ B3 (API client + repos)   impure boundary             │
                ├─ B4 (EbookBloc)       pure ← B2                   │
                ├─ B5 (WebReaderBloc)   pure ← B2                   │
                 └─ B6–B9 (screens)      impure ← B4, B5
                      ├─ B10 (E2E test)
                      ├─ B11 (webview progress)
                      └─ B12 (batch ops)
```

---

## Definition of Done (Phase 1)

- [ ] `cargo test --workspace` passes (all unit + SQLite integration tests)
- [ ] PostgreSQL integration tests pass (`cargo test -p infrastructure -- --ignored`)
- [ ] `docker compose up --build` starts successfully; health check responds 200
- [ ] OpenAPI spec at `/api/v1/system/openapi` is valid JSON with all endpoints present
- [ ] No `try/catch` / `panic!` / `unwrap` outside explicit impure boundaries; startup/config errors are typed (no panic-based flow)
- [ ] No `try/catch` outside repository implementations (Flutter)
- [ ] `flutter test` passes (all widget tests)
- [ ] Integration test B10 passes against SQLite backend
- [ ] Device ID reuse/delink semantics validated by integration tests
- [ ] SQLite↔PostgreSQL round-trip portability test passes
- [ ] URL/DOM web progress updates persisted correctly
- [ ] Batch operations (import/update/copy metadata) pass backend + frontend tests
- [ ] `CONTEXT.md` updated to reflect Phase 1 complete
