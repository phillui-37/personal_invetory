# Project Context: Personal Inventory System

## Last Updated
2026-04-18 (Phase 3 complete)

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
- **Phase 4**: Ecosystem integrations (BookWalker, Kindle, Steam/DLSite/FANZA).
- **Phase 5**: Optimization and hardening.

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
- Phase 3 planning direction is set: add image, video, and game as end-to-end vertical slices over the existing backend/frontend architecture.
- Android/iOS/desktop frontend build setup is explicitly deferred to Phase 4 as cross-cutting platform work, not Phase 3 implementation scope.

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

## References
- Requirements: `TODO.md`
- Agent rules: `AGENTS.md`
- Phase 1 tasks: `tasks/phase1.md`
- Requirements matrix: `tasks/phase1-requirements-matrix.json`
