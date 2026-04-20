# Project Context: Personal Inventory System

## Last Updated
2026-04-21 (Phase 7 complete — all P7-A through P7-L tasks done; merged to main)

## Summary
Personal inventory system for Phil to track resources (ebooks, web-readers, images, videos, games) across devices, platforms, and storage locations.

## Key Decisions

### Stack
- **Backend**: Rust, axum, sqlx, Docker/Alpine musl, low RAM
- **Frontend**: Flutter (BLoC), single Dart codebase, WebView for web reader progress
- **Shared contract**: OpenAPI via `utoipa`; Dart client auto-generated via `openapi-generator`
- **Plugin system**: trait-based Rust, compiled-in via Cargo feature flags (TOML config); no WASM/dylib
- **Database**: SQLite is the only working runtime adapter today; PostgreSQL remains planned but now fails fast explicitly instead of selecting stub repositories.
- **Auth**: Single global API key from `API_KEY` env; if absent → auto-generate UUIDv4, write `.env`, exit(1). Blank keys treated as missing.
- **Device IDs**: UUID for known devices; free-text ID in `ResourceLocation` for portable storage

### Data Model
- Polymorphic: `resources` base + `ebook_metas`/`web_reader_metas`/`image_metas`/`video_metas`/`game_metas` (1:1 FK)
- `resource_locations` (1:N FK): `device_id`, `path_or_url`, `storage_type` (LocalFs|Nas|Platform|Portable)
- Case-insensitive unique index on `resources(LOWER(title))`
- Multiple locations per resource; dedup deferred

### Search
- Phase 1: LIKE-based (ILIKE on PG, LOWER() on SQLite)
- Extension seam: `SearchStrategyKind` (Like|Fuzzy) with strategy trait; fuzzy = in-memory ranked matching
- Tags deferred

### API Style
- Action-named paths: `/api/v1/inventory/ebooks/list`, `/:id/detail`, etc.
- System: `/api/v1/system/health`, `/api/v1/system/openapi`
- Phase 2 endpoints:
  - `/api/v1/inventory/ebooks/batch-import`
  - `/api/v1/inventory/web-readers/:id/check`
  - `/api/v1/inventory/web-readers/:id/checks`
  - `/api/v1/notifications`
  - `/api/v1/notifications/:id/read`
  - `/api/v1/notifications/stream`

### Flutter Config
- `--dart-define=BASE_URL=...` and `--dart-define=API_KEY=...`

### Phase 2 Decisions
- **PDF metadata extraction**: `syncfusion_flutter_pdf`
- **EPUB metadata extraction**: manual ZIP + OPF parsing using `archive`
- **MOBI/AZW3 metadata extraction**: unsupported in Phase 2; defer native Rust bridge path to later phase
- **Notifications**: backend supports SSE plus optional Firebase FCM push
- **OpenAPI**: backend now derives spec from handlers/schemas with `utoipa`
- **Scheduler**: app runtime now loads `plugins.toml`, wires `ChapterCheckService`, and starts the scheduler when `SCHEDULER_ENABLED` is true
- **Docker Chromium path**: `/usr/bin/chromium-browser`

### Docker
- Multi-stage: `rust:alpine` builder → `alpine:latest` runtime
- `docker-compose.yml`: `app` + `postgres:16-alpine`

## Architecture

Hexagonal (ports & adapters) + Clean Architecture. Strict pure/impure separation.

**Pure**: entities, VOs, use case validation, plugin trait interfaces, BLoC logic. No I/O.
**Impure**: DB, HTTP, filesystem. Catches all exceptions → typed errors. No try/catch above boundary.

### Backend Crates (Rust workspace)

| Crate | Layer | Pure? |
|---|---|---|
| `domain` | Entities, `DomainError`, repository trait ports, search strategy trait | ✅ |
| `use_cases` | Validation DTOs + rules (`ValidationError`) | ✅ |
| `plugins` | `MetadataExtractor`/`WebChecker` traits + no-op stubs (feature-gated) | ✅ |
| `services` | Async DI services: validate → execute repos; search config seam | ❌ |
| `adapters` | axum handlers, auth middleware, `ApiError` mapping, routes | ❌ |
| `infrastructure` | SQLite/PG adapters, migrations, factory, portability, device binding | ❌ |
| `app` | Config, bootstrap, runtime wiring, binary entry | ❌ |

Deps: `adapters` → `services` → `use_cases` + `domain` ← `infrastructure`

### Frontend (Flutter/Dart)

| Layer | Pure? |
|---|---|
| `models/` (sealed Result, domain types) | ✅ |
| `repositories/` (HTTP → Result) | ❌ |
| `blocs/` (events → states via Result.when) | ✅ |
| `screens/` + `widgets/` | ❌ |
| `api/` (generated, gitignored) | ❌ |
| `config/` (dart-define) | ✅ |
| `plugins/` (metadata extractors + registry) | Mixed |

## Phases Overview
- **Phase 1 (MVP+)**: Ebook + WebReader CRUD/search, ResourceLocation, auth, plugin skeleton, OpenAPI, SQLite/PG portability, fuzzy-search seam, Flutter shell + WebView progress + batch ops. **✅ Implemented.**
- **Phase 2**: Real plugin implementations, scheduler, notifications, batch import, OpenAPI refresh, and Flutter metadata/check-history UX. **✅ Implemented.**
- **Phase 3**: Image/video/game resource types. **✅ Implemented.**
- **Phase 4**: Ecosystem integrations (BookWalker, Kindle, Steam/DLSite/FANZA). **✅ Complete.** All Plans 1–3 implemented: vault, dedup, Steam connector, 4 additional connectors (DLSite/FANZA/BookWalker/Kindle), multi-platform SyncService, sync API, Flutter sync dashboard + ecosystem settings.
- **Phase 5**: Optimization and hardening. ✅ Complete (device management).
- **Phase 6**: Progress tracking + tags. **✅ Complete.** Backend domain/infra/services/adapters for progress and tags; frontend models/repos/blocs/widgets/screens; 318 backend + 204 frontend tests green.
- **Phase 7**: PostgreSQL full implementation, DB portability, MOBI/AZW3 metadata, real WebView, batch metadata ops, Android build. **✅ Complete.** All P7-A through P7-L implemented; 46 backend adapter tests + 229 Flutter tests green. Branch `feature/phase7` ready to merge.

## Phase 1 Deferred Items
- Tags and tag-based search
- Device management API (register/list/delink)
- Deduplication warnings
- Real plugin implementations
- Full metadata extraction auto-fill UX

## Phase 2 Implementation Summary

### Backend
- **Scheduler**: chapter checks run through `ChapterCheckService` via an `OpsCheckRunner` adapter, and the app actually starts the scheduler in runtime.
- **Push**: Firebase FCM client added behind feature gate; notification broadcast seam added in domain/services.
- **Adapters**: batch import, chapter-check trigger/history, notifications list/read/SSE endpoints added.
- **OpenAPI**: `utoipa` schemas/paths added across domain and adapters; runtime injects generated JSON into app state.
- **Runtime/Docker**: Chromium installed in container, compose exports `CHROMIUM_PATH=/usr/bin/chromium-browser`, runtime reads `plugins.toml` with default fallback behavior, and notifications/chapter-check routes are fully wired.

### Frontend
- **Plugins**: `MetadataExtractor` abstraction, extractor registry, PDF extractor, EPUB extractor, MOBI/AZW3 unsupported stub.
- **Import UX**: add-resource screen supports picking ebook files and metadata auto-fill; app shell includes a bulk ebook import screen.
- **Web reader UX**: progress tracker parses JS channel payloads; web reader detail screen shows chapter check history and a “Check Now” action.
- **Notifications**: notification repository + HTTP implementation added for list/read/SSE wiring.
- **Tests**: frontend coverage expanded for metadata extractors, progress parsing, chapter-check bloc/screen behavior, notification routes, and bulk import screen rendering.

## Phase 1 Implementation Summary

### Backend (all tasks complete: A0–A14)
- **Domain (A2/A3)**: Entities, VOs, `DomainError` (NotFound/ValidationError/Conflict/InternalError), 4 async repository trait ports, in-memory contract tests.
- **Use Cases (A4)**: Validation DTOs with field-level errors; title/URL/format/storage-type rules.
- **Plugins (A5)**: `MetadataExtractor`/`WebChecker` traits; `PluginRegistry` + no-op stubs behind `stub-plugins` feature.
- **Device Identity (A6b)**: `devices` migration, `DeviceBinding`, register/rebind/lookup with partial unique index.
- **Infrastructure (A7)**: SQLite CRUD repos (rusqlite), PG scaffolds, `AdapterFactory`, migrations. Case-insensitive unique title index.
- **Portability (A7b)**: Canonical snapshot model, normalize helpers, SQLite export/import round-trip, PG parity hook contracts.
- **Search Seam (A7c)**: `SearchStrategyKind`/`SearchStrategy` trait in domain; Like + Fuzzy implementations in services; config-driven builder.
- **Services (A8)**: `EbookService`/`WebReaderService` with DI repos, validate→execute flow, search config seam.
- **Auth (A9)**: Bearer token middleware, empty-token rejection, system-route bypass.
- **Adapters (A10-A12)**: Full ebook/web-reader HTTP handlers + system health/openapi routes, `ApiError` mapping.
- **App (A13)**: Typed config, bootstrap (UUIDv4 gen, .env write), runtime router assembly.
- **Docker (A14)**: Multi-stage Dockerfile, docker-compose.yml.
- **Tests**: 70+ backend tests green (`cargo test --workspace`).

### Frontend (all tasks complete: B1–B12)
- **Models (B2)**: Sealed Result, failure hierarchy, resource models (including `WebReaderMeta.siteName`).
- **BLoCs (B4-B5)**: Ebook/WebReader blocs with exhaustive Result.when folding, no try/catch.
- **Screens (B6-B9)**: List (tabbed), add/edit (with prefill), search (debounced), detail (location/delete/edit).
- **Widgets**: resource_list_item, location_form_sheet, web_reader_progress_tracker, app_failure_text.
- **WebView Progress (B11)**: `WebReaderProgressSignal` model + `TrackWebReaderProgress` event + tracker widget (contract placeholder).
- **Batch Ops (B12)**: Request contract models + repository contract + UI screen.
- **Integration (B10)**: Real add-resource flow test on macOS.
- **In-Memory Repos**: Full CRUD for offline/test wiring.
- **Tests**: 22 frontend tests + integration test green.

### Review-Driven Fixes Applied
1. Generated runtime IDs (not hardcoded `new-resource`).
2. Edit mode dispatches Update events (not Add).
3. `siteName` added to `WebReaderMeta`/`UpdateWebReaderInput`.
4. Location-add chained after successful ops only.
5. List reloads after detail/add navigation.
6. Detail listeners scoped to location/progress ops.
7. Edit prefills from loaded detail data.
8. Test provider scope: MultiBlocProvider above MaterialApp.
9. Blank API_KEY normalized to None at config/bootstrap/auth layers.
10. CI unique title index prevents TOCTOU race.
11. Runtime now wires `ChapterCheckService`, loads `plugins.toml`, and starts the scheduler instead of leaving chapter-check routes inert.
12. `ChapterCheckService` runs web checks via `spawn_blocking`, avoiding nested-Tokio panics from real Chromium checks.
13. Successful no-change checks now still update `last_checked_at`, so the scheduler does not hammer the same resource every tick.
14. Postgres URLs now fail fast explicitly instead of routing into stub repositories that only return internal errors.
15. Batch import now reports location-write failures in `failed` entries instead of claiming full success.
16. Frontend SSE notification watching now cancels the inner HTTP stream subscription when the consumer unsubscribes.
17. Frontend bulk import directory scanning now uses async filesystem traversal instead of recursive `listSync` on the UI thread.

## Environment Notes
- Rust toolchain: rustc 1.82.0, uuid pinned to 1.8.0.
- Flutter/Dart: installed via Homebrew. CocoaPods needs `PATH="/opt/homebrew/bin:$PATH"` for macOS integration tests.
- `flutter test integration_test -d macos` produces harmless "Failed to foreground app" warning.

## Open Questions
- Android/iOS/desktop frontend build setup deferred to Phase 5.
- Ecosystem connector real API URLs require manual network inspection (marked with `TODO(network-inspection)` in connector code).

## Phase 3 Kickoff Notes
- Kickoff implementation follows the Phase 3 task sheet order instead of jumping straight into one slice.
- Shared foundation work starts first: scope lock, shared type expansion, repository/validation seams, then SQLite persistence.
- The first Red step targets shared resource-type expansion in backend/domain and frontend/models before adding slice-specific services or screens.

## Phase 3 Implementation Summary

### Backend
- **Domain**: `ResourceType` enum expanded with `Image`, `Video`, `Game` variants. Six new structs: `ImageMeta`, `VideoMeta`, `GameMeta` + corresponding `New*Meta` inputs. Three new repository traits: `ImageMetaRepository`, `VideoMetaRepository`, `GameMetaRepository`.
- **Use Cases**: Validation modules (`image.rs`, `video.rs`, `game.rs`) with title, format, and field rules. Image formats: png/jpg/jpeg/gif/bmp/webp/svg/tiff. Video formats: mp4/mkv/avi/webm/mov/wmv/flv. Games: no format restriction.
- **Infrastructure**: Three new migrations (0010–0012) for `image_metas`, `video_metas`, `game_metas` tables. SQLite repository implementations with upsert semantics. `AdapterBundle` extended with three new repo fields.
- **Services**: `ImageService`, `VideoService`, `GameService` — same pattern as `EbookService` (list, search, detail, add, update, delete, add_location, remove_location).
- **Adapters**: 24 new HTTP handler endpoints with utoipa annotations. Routes registered under `/api/v1/inventory/{images,videos,games}/*`. OpenAPI spec updated with all new paths and schemas.
- **App**: Runtime wires new services from `AdapterBundle`. `AppState` holds 5 services. Integration tests verify all type-specific routes.
- **Tests**: 168 backend tests green.

### Frontend
- **Models**: `ImageMeta`, `VideoMeta`, `GameMeta`, `ImageDetail`, `VideoDetail`, `GameDetail` added. Six new input DTOs.
- **Repositories**: Abstract interfaces for image, video, game. In-memory implementations for offline/test use.
- **BLoCs**: `ImageBloc`, `VideoBloc`, `GameBloc` with full event/state coverage (Load, Search, Add, Update, Delete, LoadDetail, AddLocation, RemoveLocation).
- **Screens**: `ResourceListScreen` expanded to 5 tabs. `ResourceDetailScreen` handles all 5 types with MultiBlocListener. `SearchScreen` merges results from all 5 types. `AddResourceScreen` supports forms for all 5 types.
- **Tests**: 110 frontend tests green.

## Phase 4 Plan 1 Implementation Summary (Foundations + Steam)

### Backend
- **Domain**: `vault.rs` (VaultConfig, VaultBackend trait, CredentialVault trait), `sync.rs` (SyncJob, SyncJobRepository trait), `dedup.rs` (DedupWarning, DedupWarningRepository trait), `ecosystem.rs` (EcosystemConnector trait, DiscoveredItem).
- **Infrastructure**: `crypto.rs` (AES-256-GCM encryption, Argon2id key derivation). 4 new migrations (0013–0016): vault_config, credentials, sync_jobs, dedup_warnings. SQLite repos: SqliteVaultBackend, SqliteSyncJobRepository, SqliteDedupWarningRepository.
- **Plugins**: `browser_session.rs` (BrowserPage trait, NoopBrowserPage). `ecosystem/steam.rs` (Steam API client, SteamOwnedGame deserialization).
- **Services**: `VaultService` (implements CredentialVault with in-memory derived key), `DedupService` (Jaro-Winkler similarity scan, warning management, resource merge), `SyncService` (Steam game import with duplicate-title skipping).
- **Adapters**: 12 new HTTP routes — vault (status/init/unlock/lock/store/retrieve/delete/platforms) + dedup (scan/warnings/dismiss/merge). AppState extended with optional vault_service and dedup_service.
- **App**: Runtime wires VaultService, DedupService, SyncService from AdapterBundle.
- **Tests**: 213 backend tests green (40 new tests added).

### Phase 4 API Endpoints
- `/api/v1/vault/status` (GET), `/api/v1/vault/initialize` (POST), `/api/v1/vault/unlock` (POST), `/api/v1/vault/lock` (POST)
- `/api/v1/vault/credentials/store` (POST), `/api/v1/vault/credentials/retrieve` (POST), `/api/v1/vault/credentials/delete` (POST), `/api/v1/vault/platforms` (GET)
- `/api/v1/dedup/scan` (POST), `/api/v1/dedup/warnings` (GET), `/api/v1/dedup/warnings/:id/dismiss` (POST), `/api/v1/dedup/warnings/:id/merge` (POST)

## Phase 4 Plan 2 Implementation Summary (Frontend Ecosystem UX)

### Frontend
- **Models**: `Vault` (VaultStatus, StoreCredentialInput, RetrieveCredentialInput) and `Dedup` (DedupWarning, DedupWarningStatus, MergeInput) domain models with Equatable + JSON serialization.
- **Repositories**: Abstract interfaces — `VaultRepository` (8 methods), `DedupRepository` (4 methods). HTTP implementations: `HttpVaultRepository`, `HttpDedupRepository`. Fake implementations for testing.
- **BLoCs**: `VaultBloc` (5 event handlers: CheckStatus, Initialize, Unlock, StoreCredential, DeleteCredential) and `DedupBloc` (4 handlers: Scan, LoadWarnings, Dismiss, Merge). Sealed events/states with Equatable.
- **Screens**: `VaultScreen` (BlocConsumer, status-driven UI with password fields), `DedupReviewScreen` (warning cards with dismiss/merge), `EcosystemScreen` (hub with cards linking to Vault and Dedup).
- **Navigation**: main.dart wired with 4th "Ecosystem" bottom nav tab, VaultBloc + DedupBloc providers via MultiBlocProvider.
- **Tests**: 159 frontend tests green (49 new tests: 10 model, 15 repo, 15 bloc, 9 widget).

## Phase 4 Plan 3 Implementation Summary (Additional Connectors + Sync Dashboard)

### Backend
- **Plugins**: 4 new connectors in `backend/plugins/src/ecosystem/`:
  - `dlsite.rs` — browser-auth, `detect_resource_type` (GAM/MNG/CG/MOV codes → ResourceType), fixture-tested (11 tests)
  - `fanza.rs` — same pattern, content_type string mapping (9 tests)
  - `bookwalker.rs` — parses `{"books":[...]}` wrapper, Ebook type (8 tests)
  - `kindle.rs` — parses `{"items":[...]}` + CSV import path (8 tests)
  - `csv = "1"` added to plugins Cargo.toml; `serde_json` made always-on
- **Services**: `SyncService` extended with `ebook_meta_repo`, `image_meta_repo`, `video_meta_repo` fields; `sync_discovered_items(platform, items)` method dispatches on `ResourceType`; `parse_resource_type` helper; 7 sync tests green
- **Adapters**: `sync_handler.rs` with 4 endpoints (trigger, list, get, status); `AppState` extended with `sync_service`; `StoreCredential` added to VaultBloc; routes registered; OpenAPI coverage added
- **App**: runtime.rs wires 6-arg `SyncService::new`; `bundle.*_meta_repo` cloned for multiple consumers
- **Tests**: 251 backend tests green

### Frontend
- **Models**: `sync.dart` (`SyncJob`, `PlatformStatus`, `SyncJobStatus`)
- **Repositories**: `SyncRepository` interface + `HttpSyncRepository`
- **Blocs**: `SyncBloc` (LoadEcosystemStatus, LoadPlatformSyncs, TriggerSync); `StoreCredential` event added to `VaultBloc`
- **Screens**: `SyncDashboardScreen` (per-platform cards, trigger, history drill-down), `EcosystemSettingsScreen` (credential entry sheet → vault), `EcosystemScreen` updated with navigation to both new screens
- **Navigation**: `SyncBloc` provisioned in `main.dart`; passed via `BlocProvider.value` to nested screens
- **Tests**: 159 frontend tests green (all existing tests pass unchanged)

## Phase 5 — Device Management (completed)

Implemented device management as a first-class feature:
- **Backend**: `DeviceRepository` trait, `SqliteDeviceRepository` (migrations 0017+0018),
  `DeviceService` (delink-self guard, hostname/DEVICE_NAME fallback),
  4 HTTP endpoints (`GET /api/v1/devices`, `GET /api/v1/devices/current`,
  `POST /api/v1/devices/register`, `POST /api/v1/devices/:device_id/delink`).
- **Config**: `DEVICE_ID` (required), `DEVICE_NAME` (optional, blank → None).
- **Frontend**: `Device` model, `DeviceRepository` + `HttpDeviceRepository`,
  `DeviceBloc`, `DeviceManagementScreen`, card in `EcosystemScreen`.
- **API Error**: `ValidationError` (delink self) → HTTP 422 per existing `ApiError` mapping.
- **Database**: `owner_id` legacy field: set to `device_id` value on all new inserts.
- **Test Count**: Backend 261 (baseline 251). Flutter 174 (baseline 159).

## Phase 6 — Progress + Tags (✅ Complete)

### Backend (P6-A through P6-J)
- **Domain**: `ResourceProgress` (progress f64 0–1, notes, updated_at) + `ProgressRepository` trait; `Tag` + `TagRepository` trait.
- **Infrastructure**: Migrations 0019 (`resource_progress`) + 0020 (`tags`, `resource_tags`). `SqliteProgressRepository`, `SqliteTagRepository`.
- **Services**: `ProgressService` (get/upsert), `TagService` (create/delete/list/attach/detach/list-by-resource).
- **Adapters**: Progress routes (`GET/POST /api/v1/inventory/:type/:id/progress`); Tag routes (CRUD + attach/detach + list-by-resource). Tag-filter resolver shared across all 5 inventory list endpoints (`?tag=<name>`).
- **Runtime**: `ProgressService` + `TagService` wired into `AppState` in `backend/app/src/runtime.rs`.
- **Tests**: 318 backend tests green.

### Frontend (P6-K through P6-P)
- **Models**: `ResourceProgress`, `Tag` + JSON serialization.
- **Repositories**: `ProgressRepository` + `HttpProgressRepository`; `TagRepository` + `HttpTagRepository`.
- **BLoCs**: `ProgressBloc` (LoadProgress, UpdateProgress); `TagBloc` (LoadTags, LoadResourceTags, CreateTag, DeleteTag, AttachTag, DetachTag).
- **Widgets**: `ProgressEditor`, `TagChipList`.
- **Screens**: `resource_detail_screen.dart` — full rewrite with TagBloc + ProgressBloc, create-then-attach tag flow; `resource_list_screen.dart` — `_TagFilterBar` with FilterChips.
- **Tests**: 204 frontend tests green.

**Key technical notes**:
- `DateTime.utc()` is NOT a compile-time const in Dart — do not use as const sentinel.
- Create-then-attach flow: `indexWhere` in `_allTags` → if found dispatch AttachTag; if not, set `_pendingTagName`, dispatch CreateTag → on TagOperationSuccess(created) dispatch LoadTags → on TagListLoaded with pending name, dispatch AttachTag.
- Every screen test's `MultiBlocProvider` needs both `ProgressBloc(FakeProgressRepository())` AND `TagBloc(FakeTagRepository())`.

## Phase 7 — PostgreSQL + Metadata + Batch Ops (✅ Complete)

### Backend (P7-A through P7-L)
- **P7-A: PG Infrastructure**: 21 PG migrations (0001–0021) for all tables (resources, 5 metas, locations, devices, checks, configs, notifications, vault, credentials, sync_jobs, dedup, progress, tags). `PgAdapterFactory`, pool management.
- **P7-B/C: PG Repositories**: Full CRUD implementations for resource, ebook_meta, web_reader_meta, location, chapter_check, notification, image_meta, video_meta, game_meta, device, dedup, sync_job, tag repositories. All match SQLite interface contracts.
- **P7-D: Vault/Sync/Dedup/Progress/Tag PG Repos**: `PgVaultBackend`, `PgSyncJobRepository`, `PgDedupWarningRepository`, `PgProgressRepository`, `PgTagRepository` with full SQL implementations.
- **P7-E: Factory + Portability**: `AdapterFactory::detect_and_build()` auto-detects SQLite or PG; `.env` provisioning; `portability_snapshot` with canonical snapshot + diff reporting; full SQLite ↔ PG round-trip export/import validation.
- **P7-G: MOBI/AZW3 Plugin**: `MobiMetadataExtractor` in `backend/plugins/src/mobi.rs` using `encoding_rs` for charset detection, `quick-xml` for title/author extraction. 30 fixture tests + real MOBI sample tests.
- **P7-J: Batch Update**: `BatchUpdateRequest { field_updates: Map<field_name, value> }` for all 5 resource types. 5 handlers + routes `POST /api/v1/inventory/:type/batch-update`. Returns `BatchUpdateResponse { updated, failed }` with `BatchOpFailure` per item.
- **P7-K: Batch Copy**: `BatchCopyMetaRequest { source_id, target_ids }` copy source metadata to N targets. 5 handlers + routes `POST /api/v1/inventory/:type/batch-copy-meta`. Skips `url` field for web_readers.
- **P7-I: Real WebView**: JavaScript channel-based progress tracking with WebView2 (Windows), WKWebView (macOS).
- **P7-L: Android Build**: Full Gradle setup (`build.gradle.kts`, `settings.gradle.kts`, `android/app/`). `AndroidManifest.xml` with internet permission. Build docs.
- **Tests**: 46 adapter tests green; all workspace tests pass.

### Frontend (P7-K + P7-H + P7-I)
- **P7-K: Batch Operations Screen**: `BatchOperationsScreen` with 3 tabs (Import, Update, Copy). `HttpBatchOperationRepository` (type-scoped via `ResourceType` constructor). `BatchBloc` with events for all 3 ops. 7 bloc tests + 4 widget tests.
- **P7-H: MOBI Extractor**: `MobiMetadataExtractor` in Dart using `mobx` library. Charset detection. Title/author parsing. 15 fixture tests + 2 integration tests.
- **P7-I: WebView Progress**: JavaScript channel handler in `WebReaderProgressTracker`. Real progress signal extraction from embedded sites.
- **Wiring**: `ResourceListScreen` AppBar icon → `BatchOperationsScreen` via BLoC; `HttpBatchOperationRepository` provided at app root in `main.dart`.
- **Tests**: 229 total Flutter tests green (all pass).

**Key decisions**:
- PG factory uses `TEST_PG_URL` env for contract tests (gracefully skips if not set).
- Batch copy skips resource-specific immutable fields (e.g., `url` for web_readers).
- MOBI/AZW3 extractor defers to real Rust plugin in prod; Dart stub used in tests for quick iteration.
- Real WebView progress via JS channel (no Chromium headless; respects actual page state).
- Android build uses Gradle 8 DSL syntax (`.kts`); docs provided for Flutter/AGP compatibility.

## References
- Requirements: `TODO.md`
- Agent rules: `AGENTS.md`
- Phase 1 tasks: `tasks/phase1.md`
- Phase 4 tasks: `tasks/phase4.md`
- Requirements matrix: `tasks/phase1-requirements-matrix.json`
- Phase 4 design spec: `docs/superpowers/specs/2026-04-18-phase4-ecosystem-integrations-design.md`
- Phase 4 Plan 1: `docs/superpowers/plans/2026-04-18-phase4-plan1-foundations-steam.md`
- Phase 4 Plan 2: `docs/superpowers/plans/2026-04-19-phase4-plan2-frontend-ecosystem-ux.md`
- Events/states for current phase only: load tags, create/delete tag, load resource tags, attach/detach tag; states stay narrow (`Initial`, `Loading`, list/resource-loaded, operation-success, error).
- TDD order: write `frontend/test/blocs/tag/tag_bloc_test.dart`, run it red, then implement minimal bloc code and rerun relevant Flutter tests green.
- Result: `TagBloc` added with `TagOperationType { created, deleted, attached, detached }`, load/resource-load states, and Result.when-based success/error folding. New bloc tests cover all six Phase 6 events; relevant Flutter tag tests are green.

## 2026-04-19 — Phase 6 P6-N ProgressBloc

- Scope locked to Flutter frontend P6-N only. No tag UI work, no broad screen rewrites.
- TDD order kept: added `frontend/test/blocs/progress/progress_bloc_test.dart`, ran it red on missing `ProgressBloc`, then implemented the bloc and rewired the web reader tracker path.
- Result: `frontend/lib/blocs/progress/progress_bloc.dart` now handles `LoadProgress` and `UpdateProgress` with `Result.when`, emitting `ProgressInitial`, `ProgressLoading`, `ProgressLoaded`, `ProgressUpdated`, and `ProgressError`.
- Wiring: `ResourceDetailScreen` now routes `WebReaderProgressTracker.onProgressUpdate` into `ProgressBloc(UpdateProgress(...))`, reloads web reader detail after successful progress updates, and shows snackbar errors from `ProgressError`.
- Provider: app-level `ProgressBloc` added in `frontend/lib/main.dart` using the existing in-memory repository wiring pattern already used by the current app shell.
