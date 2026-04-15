# Project Context: Personal Inventory System

## Last Updated
2026-04-15 (Phase 1 design brainstorming complete; spec written)

## Summary
Building a personal inventory system for Phil to track resources (images, ebooks, videos, games) across devices, platforms, and storage locations.

## Key Decisions

### Stack
- **Backend**: Rust, axum (HTTP), sqlx (DB), containerized (Docker/Alpine musl), low RAM
- **Frontend**: Flutter — single Dart codebase, production-ready WebView on desktop + mobile, embedded webview for web reader progress tracking.
- **Shared contract**: OpenAPI spec auto-generated from backend via `utoipa`
- **Plugin system**: conf-based (TOML/YAML), trait-based in Rust, compiled-in via Cargo feature flags; OCP enforced. No dynamic loading (no WASM/dylib).
- **Database**: PostgreSQL (primary) + SQLite (portable/local); shared schema with adapter factory; no PostgreSQL-only types in schema (ensures SQLite compatibility)
- **Auth**: Single global API key from `API_KEY` env var; if absent → auto-generate UUIDv4, write to `.env` (best-effort), echo warning, exit(1)
- **Device IDs**: auto-generated UUID for known devices; portable storage (USB, external HDD, etc.) uses a free-text ID field in `ResourceLocation` — no special device type needed.

### Data Model
- Polymorphic: shared `resources` base table + separate `ebook_metas` and `web_reader_metas` tables (1:1 FK)
- `resource_locations` (1:N FK to resources): stores `device_id` (UUID string or free-text), `path_or_url`, `storage_type` (LocalFs | Nas | Platform | Portable)
- Multiple locations per resource supported from day one; dedup logic deferred

### Search
- Phase 1: `LIKE`-based (`ILIKE` on PostgreSQL, `LOWER()` on SQLite)
- Later phases: string-similarity % checking
- Tags deferred to a later phase

### API Style
- Action-named, type-segregated verbose paths: `/api/v1/inventory/ebooks/list`, `/api/v1/inventory/ebooks/:id/detail`, etc.
- Full separation between ebook and web-reader routes

### Flutter Config
- Backend URL + API key passed via `--dart-define=BASE_URL=...` and `--dart-define=API_KEY=...`
- API client auto-generated from OpenAPI spec using `openapi-generator` (Dart target)

### Docker
- Multi-stage: `rust:alpine` builder (musl static binary) → `alpine:latest` runtime
- Clear all cargo/build caches in builder; `apk cache clean` in runtime
- `docker-compose.yml`: `app` + `postgres:16-alpine`; SQLite mode = override `DATABASE_URL`

## Architecture Style
Hexagonal (ports & adapters) + Clean Architecture layers. Strict pure/impure separation:

**Pure** — entities, value objects, use case functions, plugin trait interfaces, BLoC logic. No I/O, no async, no exceptions. Validation errors are acceptable as typed return values (not exceptions).

**Impure** — all I/O: DB, HTTP, file system, user interaction. Impure boundary catches all exceptions and converts to typed errors. Nothing above the boundary uses try/catch or propagates raw exceptions.

### Backend Crate Structure (Rust)

| Crate | Layer | Pure? |
|---|---|---|
| `domain` | Entities + repository/plugin trait ports | ✅ Pure |
| `use_cases` | Pure use case functions (validation, orchestration) | ✅ Pure |
| `plugins` | Plugin trait interfaces + no-op stubs | ✅ Pure |
| `services` | Async executors: call use_cases + inject repo traits + map IO errors | ❌ Impure boundary |
| `adapters` | HTTP handlers (axum), DTOs, request/response mappers, utoipa | ❌ Impure |
| `infrastructure` | sqlx DB adapters, plugin implementations | ❌ Impure |
| `app` | Binary: config, DI wiring, startup, API key bootstrap | ❌ Impure |

Dependency direction: `adapters` → `services` → `use_cases` + `domain` ← `infrastructure`

### Flutter Structure (Dart)

Pattern: **BLoC** (flutter_bloc)  
Error propagation: **Dart 3 sealed `Result<T>`** — Repository catches all exceptions, returns typed `Result`; BLoC exhaustively switches on it; no try/catch above Repository layer.

| Layer | Pure? | Responsibility |
|---|---|---|
| `models/` | ✅ Pure | Sealed domain types, sealed `Result<T, Failure>` |
| `repositories/` | ❌ Impure | HTTP calls via generated client; catch exceptions → return `Result` |
| `blocs/` | ✅ Pure logic | Receive events, fold `Result` → state; no I/O, no try/catch |
| `screens/` + `widgets/` | ❌ Impure | UI rendering, user interaction dispatch |
| `api/` | ❌ Impure | Generated OpenAPI client (gitignored) |
| `config/` | ✅ Pure | `AppConfig` reading dart-define values |

## Current Phase
**Phase 1 — MVP design approved. Spec written. Awaiting task file + implementation plan.**

## Phases Overview
- **Phase 1 (MVP)**: Ebook + WebReader CRUD/search, ResourceLocation, auth, plugin skeleton, OpenAPI, Flutter minimal shell (list/add/search/detail — no WebView)
- **Phase 2**: Plugin system (scrapers: DLSite, FANZA, Steam; metadata extractors; web chapter checkers)
- **Phase 3**: Advanced frontend (WebView + progress tracking, batch ops, metadata auto-fill)
- **Phase 4**: Platform integrations (BookWalker, Kindle, Steam library)
- **Phase 5**: Data portability (PostgreSQL ↔ SQLite migration, export/import)

## Phase 1 Deferred Items
- Tags and tag-based search
- Device management API (register/list/delink devices)
- Progress tracking (comes with WebView in Phase 3)
- Deduplication warnings
- Resource types: image, video, game
- Real plugin implementations (scrapers, metadata extractors)
- String-similarity fuzzy search

## Open Questions
_(All resolved — none remaining.)_

## References
- Requirements: `TODO.md`
- Agent rules: `AGENTS.md`
- Phase 1 spec: `docs/superpowers/specs/2026-04-15-phase1-mvp-design.md`
- Phase 1 tasks: `tasks/phase1.md` (pending)
