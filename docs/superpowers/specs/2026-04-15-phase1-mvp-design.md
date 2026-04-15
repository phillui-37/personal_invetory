# Phase 1 MVP — Design Spec

**Date:** 2026-04-15  
**Status:** Approved

---

## Problem Statement

Phil owns a large number of ebooks and web-reader resources (manga, light novels, web serials) spread across multiple devices and storage locations. There is no single place to track what he owns, where it is stored, or its metadata. Phase 1 delivers the minimal viable system: a backend API + thin Flutter shell that lets Phil add, browse, search, and view details for ebooks and web-reader resources across locations.

---

## Scope

**In scope (Phase 1):**
- Resource types: **Ebook** and **WebReader** only
- Full CRUD for resources + type-specific metadata
- Multiple `ResourceLocation` entries per resource (device_id + path/URL + storage_type)
- LIKE-based search on title, author (ebooks), URL/site name (web-readers)
- Single global API key auth (from env, auto-generated on first start)
- Plugin skeleton: trait definitions + no-op stubs (no real plugins yet)
- OpenAPI spec auto-generated via `utoipa`, served at `/api/v1/system/openapi`
- Flutter minimal shell: list, add, search, detail (no WebView, no progress tracking)
- Docker: Alpine-based multi-stage image + docker-compose with PostgreSQL
- Both PostgreSQL and SQLite supported via adapter factory

**Deferred to later phases:**
- Tags, tag-based search, string-similarity fuzzy search
- Device management API (devices are just free-text IDs on locations for now)
- Progress tracking (comes with WebView in Phase 3)
- Deduplication warnings
- All other resource types (image, video, game)
- Real plugin implementations (scrapers, metadata extractors)

---

## Architecture

Hexagonal (ports & adapters). The `domain` crate has no I/O and defines all entities, value objects, and repository traits. Infrastructure (DB, HTTP) depends on domain; domain never depends on infrastructure.

```
personal_inventory/
├── backend/
│   ├── domain/        # entities, repository traits, domain errors (no I/O)
│   ├── db/            # sqlx adapters for PostgreSQL + SQLite
│   ├── http/          # axum HTTP handlers, middleware, utoipa OpenAPI
│   ├── plugins/       # plugin traits + no-op stubs
│   └── app/           # binary: wires domain + db + http + plugins
├── frontend/          # Flutter project
├── docker/
│   ├── Dockerfile
│   └── docker-compose.yml
└── tasks/
    └── phase1.md
```

---

## Domain Model

### `Resource` (base entity)
| Field | Type | Notes |
|---|---|---|
| `id` | `Uuid` | PK |
| `title` | `String` | required |
| `notes` | `Option<String>` | free-text |
| `resource_type` | `ResourceType` | `Ebook` \| `WebReader` |
| `created_at` | `DateTime<Utc>` | |
| `updated_at` | `DateTime<Utc>` | |

### `EbookMeta` (1:1 FK to Resource)
| Field | Type |
|---|---|
| `resource_id` | `Uuid` |
| `author` | `Option<String>` |
| `isbn` | `Option<String>` |
| `publisher` | `Option<String>` |
| `language` | `Option<String>` |
| `file_format` | `Option<String>` — pdf/epub/mobi/azw3 |

### `WebReaderMeta` (1:1 FK to Resource)
| Field | Type |
|---|---|
| `resource_id` | `Uuid` |
| `url` | `String` — required |
| `site_name` | `Option<String>` |
| `last_checked_chapter` | `Option<String>` |

### `ResourceLocation` (1:N FK to Resource)
| Field | Type | Notes |
|---|---|---|
| `id` | `Uuid` | PK |
| `resource_id` | `Uuid` | FK |
| `device_id` | `String` | UUID string for known devices; free-text for portables |
| `path_or_url` | `String` | file path or URL |
| `storage_type` | `StorageType` | `LocalFs` \| `Nas` \| `Platform` \| `Portable` |

### Repository Traits (in `domain`)
- `ResourceRepository` — CRUD + LIKE search
- `EbookMetaRepository` — get/upsert by resource_id
- `WebReaderMetaRepository` — get/upsert by resource_id
- `LocationRepository` — list by resource_id, add, remove

---

## Database

### Schema rules (PostgreSQL + SQLite compatible)
- No `JSONB`, `ARRAY`, or PostgreSQL-only types
- Use `TEXT` for UUIDs (stored as UUID strings)
- Use `TEXT` for `DateTime` (ISO 8601 strings)
- Use `INTEGER` (0/1) for booleans if needed
- sqlx migrations in `db/migrations/`

### Adapter factory
At startup, `DATABASE_URL` prefix determines which adapter is used:
- `postgres://` → PostgreSQL adapters
- `sqlite://` → SQLite adapters

Both implement the same repository traits.

---

## API Layer

**Auth:** Every request (except `/api/v1/system/health`) requires `Authorization: Bearer <API_KEY>` header.

**API key bootstrap:**
1. Read `API_KEY` from env
2. If absent → generate `UUIDv4`, attempt to write to `.env` file (best-effort; skip if not writable, e.g. in Docker read-only mounts), print `⚠️ WARNING: No API_KEY found. Generated key: <key>. Set API_KEY=<key> in your environment and restart.` then exit(1).

**Endpoints:**

```
# Ebooks
GET  /api/v1/inventory/ebooks/list
GET  /api/v1/inventory/ebooks/search?q=<term>
POST /api/v1/inventory/ebooks/add
GET  /api/v1/inventory/ebooks/:id/detail
PUT  /api/v1/inventory/ebooks/:id/update
DELETE /api/v1/inventory/ebooks/:id/delete
POST /api/v1/inventory/ebooks/:id/locations/add
DELETE /api/v1/inventory/ebooks/:id/locations/:loc_id/remove

# Web Readers
GET  /api/v1/inventory/web-readers/list
GET  /api/v1/inventory/web-readers/search?q=<term>
POST /api/v1/inventory/web-readers/add
GET  /api/v1/inventory/web-readers/:id/detail
PUT  /api/v1/inventory/web-readers/:id/update
DELETE /api/v1/inventory/web-readers/:id/delete
POST /api/v1/inventory/web-readers/:id/locations/add
DELETE /api/v1/inventory/web-readers/:id/locations/:loc_id/remove

# System (no auth)
GET  /api/v1/system/health
GET  /api/v1/system/openapi   → returns openapi.json
```

**OpenAPI:** auto-generated via `utoipa` annotations on handler functions. Served as JSON.

**LIKE search:** `SELECT ... WHERE title LIKE '%<q>%' OR author LIKE '%<q>%'` (ebooks); `WHERE title LIKE '%<q>%' OR url LIKE '%<q>%' OR site_name LIKE '%<q>%'` (web-readers). Case-insensitive via `ILIKE` on PostgreSQL, `LIKE` + `LOWER()` on SQLite.

---

## Plugin Skeleton

```rust
// In plugins crate

pub trait MetadataExtractor: Send + Sync {
    fn resource_type(&self) -> ResourceType;
    fn extract(&self, input: &str) -> Result<ExtractedMeta, PluginError>;
}

pub trait WebChecker: Send + Sync {
    fn check(&self, url: &str) -> Result<CheckResult, PluginError>;
}

// Stub (compiled in with feature "stub-plugins", on by default in dev)
pub struct NoOpMetadataExtractor;
pub struct NoOpWebChecker;
```

Plugin registry holds `Vec<Box<dyn MetadataExtractor>>` and `Vec<Box<dyn WebChecker>>`. TOML config (e.g. `plugins.toml`) lists enabled plugins by name. In Phase 1, only the no-op stubs exist.

---

## Docker

**Dockerfile** (multi-stage):
1. `FROM rust:alpine AS builder` — install musl tools, copy source, `cargo build --release --target x86_64-unknown-linux-musl`, clean cargo cache and target intermediates.
2. `FROM alpine:latest AS runtime` — copy binary only, `apk cache clean`, run as non-root user.

**docker-compose.yml:**
- `app` service: reads `DATABASE_URL`, `API_KEY` from `.env`; depends on `postgres`; health check on `/api/v1/system/health`.
- `postgres` service: official `postgres:16-alpine`; volume for data persistence.
- SQLite mode: override `DATABASE_URL=sqlite://./data/inventory.db`, no postgres service needed.

---

## Flutter Minimal Shell

**API client:** generated from OpenAPI spec using `openapi-generator` (Dart target). Regenerated whenever the backend spec changes.

**Config:** backend base URL and API key via `--dart-define=BASE_URL=http://...` and `--dart-define=API_KEY=...` at build time (or injected via CI env).

**Screens:**

| Screen | Features |
|---|---|
| Resource List | Tabbed (Ebooks / Web Readers), title + storage type badge, pull-to-refresh |
| Add Resource | Type selector, common fields, type-specific fields, inline location add |
| Search | Search bar, LIKE results list, tap to detail |
| Resource Detail | All fields read-only, location list, edit + delete actions |

**No WebView, no progress tracking in Phase 1.**

---

## Testing Strategy (TDD)

- `domain` crate: pure unit tests (no I/O, fast)
- `db` crate: integration tests against real SQLite (in-memory) and PostgreSQL (via Docker in CI)
- `http` crate: integration tests using `axum::test` with mock repositories
- `plugins` crate: unit tests for stub no-op behaviour
- Flutter: widget tests for each screen; integration test for add-resource flow

---

## Error Handling

- Domain errors: typed enum `DomainError` (NotFound, ValidationError, etc.)
- HTTP errors: mapped to appropriate HTTP status codes + JSON body `{ "error": "..." }`
- DB errors: wrapped and mapped in adapter layer; never leak raw sqlx errors to domain
- Plugin errors: logged, never propagate to API response (graceful degradation)
