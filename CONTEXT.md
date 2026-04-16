# Project Context: Personal Inventory System

## Last Updated
2025-04-17 (Phase 1 implementation complete, pending commit)

## Summary
Personal inventory system for Phil to track resources (ebooks, web-readers; images/videos/games deferred) across devices, platforms, and storage locations.

## Key Decisions

### Stack
- **Backend**: Rust, axum, sqlx, Docker/Alpine musl, low RAM
- **Frontend**: Flutter (BLoC), single Dart codebase, WebView for web reader progress
- **Shared contract**: OpenAPI via `utoipa`; Dart client auto-generated via `openapi-generator`
- **Plugin system**: trait-based Rust, compiled-in via Cargo feature flags (TOML config); no WASM/dylib
- **Database**: PostgreSQL (primary) + SQLite (portable); shared schema via adapter factory; no PG-only types
- **Auth**: Single global API key from `API_KEY` env; if absent → auto-generate UUIDv4, write `.env`, exit(1). Blank keys treated as missing.
- **Device IDs**: UUID for known devices; free-text ID in `ResourceLocation` for portable storage

### Data Model
- Polymorphic: `resources` base + `ebook_metas`/`web_reader_metas` (1:1 FK)
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

### Flutter Config
- `--dart-define=BASE_URL=...` and `--dart-define=API_KEY=...`

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

## Phases Overview
- **Phase 1 (MVP+)**: Ebook + WebReader CRUD/search, ResourceLocation, auth, plugin skeleton, OpenAPI, SQLite/PG portability, fuzzy-search seam, Flutter shell + WebView progress + batch ops. **✅ Implemented.**
- **Phase 2**: Real plugin implementations (metadata extractors, chapter checkers).
- **Phase 3**: Image/video/game resource types.
- **Phase 4**: Ecosystem integrations (BookWalker, Kindle, Steam/DLSite/FANZA).
- **Phase 5**: Optimization and hardening.

## Phase 1 Deferred Items
- Tags and tag-based search
- Device management API (register/list/delink)
- Deduplication warnings
- Resource types: image, video, game
- Real plugin implementations
- Full metadata extraction auto-fill UX

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

## Environment Notes
- Rust toolchain: rustc 1.82.0, uuid pinned to 1.8.0.
- Flutter/Dart: installed via Homebrew. CocoaPods needs `PATH="/opt/homebrew/bin:$PATH"` for macOS integration tests.
- `flutter test integration_test -d macos` produces harmless "Failed to foreground app" warning.

## Open Questions
_(None remaining.)_

## References
- Requirements: `TODO.md`
- Agent rules: `AGENTS.md`
- Phase 1 tasks: `tasks/phase1.md`
- Requirements matrix: `tasks/phase1-requirements-matrix.json`
