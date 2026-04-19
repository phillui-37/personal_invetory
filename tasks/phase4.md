# Phase 4 Tasks — Ecosystem Integrations

**Spec context:** `CONTEXT.md`, `TODO.md`, `AGENTS.md`, `docs/superpowers/specs/2026-04-18-phase4-ecosystem-integrations-design.md`  
**Approach:** Backend-first, vertical slices by subsystem (vault → dedup → sync → connectors → frontend)  
**Methodology:** TDD — Red → Green → Refactor on every task  
**Scope:** Vault (credential management), dedup (duplicate detection), sync infrastructure, and ecosystem connectors (Steam, DLSite, FANZA, BookWalker, Kindle)  
**Deferred:** Android/iOS/desktop build setup belongs to **Phase 5** as cross-cutting platform work

---

## Current State

- Backend supports five resource types (Ebook, WebReader, Image, Video, Game) end to end
- No credential storage, duplicate detection, or external-platform sync existed before Phase 4
- Game resources support manual entry only — no ecosystem connector wiring
- Frontend has no vault, dedup, or ecosystem screens
- 213 backend tests green (pre-Phase 4 baseline: 173)

---

## Phase 4 Objectives

1. Add secure credential vault with AES-256-GCM encryption and Argon2id key derivation
2. Add duplicate detection engine with Jaro-Winkler similarity scoring and resource merge
3. Add sync infrastructure for importing resources from external platforms
4. Implement Steam connector as first ecosystem integration
5. Implement DLSite, FANZA, BookWalker, and Kindle connectors
6. Build frontend UX for vault management, dedup review, and ecosystem sync dashboard
7. Keep platform build scaffolding and CI pipeline work out of Phase 4

---

## Implementation Plans

| Plan | Scope | Doc |
|---|---|---|
| Plan 1 | Foundations + Steam | `docs/superpowers/plans/2026-04-18-phase4-plan1-foundations-steam.md` |
| Plan 2 | Frontend Ecosystem UX | `docs/superpowers/plans/2026-04-19-phase4-plan2-frontend-ecosystem-ux.md` |
| Plan 3+ | Additional connectors, sync dashboard, sync API | Not yet written |

---

## Phase 4 API Endpoints

### Vault
```
GET  /api/v1/vault/status
POST /api/v1/vault/initialize
POST /api/v1/vault/unlock
POST /api/v1/vault/lock
POST /api/v1/vault/credentials/store
POST /api/v1/vault/credentials/retrieve
POST /api/v1/vault/credentials/delete
GET  /api/v1/vault/platforms
```

### Dedup
```
POST /api/v1/dedup/scan
GET  /api/v1/dedup/warnings
POST /api/v1/dedup/warnings/:id/dismiss
POST /api/v1/dedup/warnings/:id/merge
```

### Sync (implemented — design spec paths)
```
POST /api/v1/ecosystem/:platform/sync
GET  /api/v1/ecosystem/:platform/syncs
GET  /api/v1/ecosystem/:platform/syncs/:id
GET  /api/v1/ecosystem/status
```

---

## Track P4-A — Roadmap Lock ✅

### P4-A1 · Phase 4 Requirement Lock ✅

Map the relevant deferred requirements from `TODO.md` and Phase 3 into this phase:
- ecosystem connectors: Steam, DLSite, FANZA, BookWalker, Kindle
- vault and credential management
- duplicate detection and merge
- sync infrastructure
- Android/iOS/desktop build setup stays deferred and moved into Phase 5 roadmap language

**Acceptance:** Phase 4 scope is explicit and Phase 5 owns platform build setup.

---

## Track P4-B — Vault & Crypto ✅

### P4-B1 · Domain — Vault Entities & Traits ✅

Define vault domain types:
- `VaultConfig` (id, master key hash, salt, created/updated timestamps)
- `VaultBackend` trait (CRUD for vault config and encrypted credentials)
- `CredentialVault` trait (initialize, unlock, lock, store, retrieve, delete, list platforms)

**Acceptance:** Domain crate compiles with vault types. No I/O in crate.

### P4-B2 · Infrastructure — AES-256-GCM Crypto Module ✅

Implement `crypto.rs`:
- AES-256-GCM encryption/decryption
- Argon2id key derivation from master password + salt
- Deterministic nonce generation for credential storage

**Acceptance:** Crypto round-trip tests pass. Key derivation produces consistent output for same inputs.

### P4-B3 · Infrastructure — Vault Migrations & SQLite Repository ✅

Add migrations and persistence:
- Migration 0013: `vault_config` table
- Migration 0014: `credentials` table (platform, encrypted data, nonce, timestamps)
- `SqliteVaultBackend` implementing `VaultBackend` trait

**Acceptance:** Migrations apply cleanly. `cargo test -p infrastructure` green.

### P4-B4 · Services — VaultService ✅

Implement `VaultService`:
- Wraps `VaultBackend` with in-memory derived key
- Initialize vault with master password (derives key, stores config)
- Unlock (verify password, hold derived key in memory)
- Lock (clear derived key)
- Store/retrieve/delete credentials (encrypt/decrypt via derived key)
- List configured platforms

**Acceptance:** `cargo test -p services` green. Vault lifecycle tests pass.

### P4-B5 · Adapters — Vault HTTP Routes ✅

Add 8 vault endpoints:
- Status, initialize, unlock, lock
- Credential store, retrieve, delete
- Platform list

**Acceptance:** `cargo test -p adapters` green. All vault routes registered and reachable.

---

## Track P4-C — Dedup Engine ✅

### P4-C1 · Domain — Dedup Entities & Traits ✅

Define dedup domain types:
- `DedupWarning` (id, resource IDs, similarity score, status, timestamps)
- `DedupWarningRepository` trait (create, list, get, dismiss)

**Acceptance:** Domain crate compiles with dedup types. No I/O in crate.

### P4-C2 · Infrastructure — Dedup Migrations & SQLite Repository ✅

Add persistence:
- Migration 0015: `dedup_warnings` table
- `SqliteDedupWarningRepository` implementing `DedupWarningRepository`

**Acceptance:** Migration applies cleanly. `cargo test -p infrastructure` green.

### P4-C3 · Services — DedupService ✅

Implement `DedupService`:
- Scan all resources using Jaro-Winkler similarity on titles
- Generate warnings above configurable threshold
- Dismiss individual warnings
- Merge duplicate resources (consolidate locations, metadata, delete duplicate)

**Acceptance:** `cargo test -p services` green. Similarity detection and merge logic tested.

### P4-C4 · Adapters — Dedup HTTP Routes ✅

Add 4 dedup endpoints:
- Scan trigger
- Warning list
- Warning dismiss
- Warning merge

**Acceptance:** `cargo test -p adapters` green. All dedup routes registered and reachable.

---

## Track P4-D — Sync Infrastructure & Steam Connector ✅

### P4-D1 · Domain — Sync & Ecosystem Entities ✅

Define sync and ecosystem domain types:
- `SyncJob` (id, platform, status, item counts, timestamps)
- `SyncJobRepository` trait (create, update status, list, get)
- `EcosystemConnector` trait (discover items from external platform)
- `DiscoveredItem` (title, platform-specific ID, metadata)

**Acceptance:** Domain crate compiles with sync types. No I/O in crate.

### P4-D2 · Infrastructure — Sync Migrations & SQLite Repository ✅

Add persistence:
- Migration 0016: `sync_jobs` table
- `SqliteSyncJobRepository` implementing `SyncJobRepository`

**Acceptance:** Migration applies cleanly. `cargo test -p infrastructure` green.

### P4-D3 · Plugins — Browser Session Abstraction ✅

Implement browser session support:
- `BrowserPage` trait (navigate, query selector, get text, screenshot)
- `NoopBrowserPage` for testing
- Foundation for DLSite/FANZA cookie-based scraping in later plans

**Acceptance:** `cargo test -p plugins` green. Trait compiles with noop impl.

### P4-D4 · Plugins — Steam Connector ✅

Implement `ecosystem/steam.rs`:
- Steam Web API client (`IPlayerService/GetOwnedGames`)
- `SteamOwnedGame` deserialization (appid, name, playtime, icon URL)
- Implements `EcosystemConnector` trait

**Acceptance:** `cargo test -p plugins` green. Steam API response deserialization tested.

### P4-D5 · Services — SyncService (Steam Import) ✅

Implement `SyncService`:
- Orchestrate Steam game import via connector + resource creation
- Duplicate-title skipping (skip games already in inventory)
- SyncJob lifecycle tracking (pending → running → completed/failed)

**Acceptance:** `cargo test -p services` green. Import with dedup skipping tested.

### P4-D6 · App — Runtime Wiring ✅

Wire Phase 4 services into application runtime:
- `AppState` extended with optional `vault_service` and `dedup_service`
- `VaultService`, `DedupService`, `SyncService` constructed from `AdapterBundle`
- All new routes registered in router

**Acceptance:** Application starts with Phase 4 services. `cargo test --workspace` green (213 tests).

---

## Track P4-E — Frontend Ecosystem UX ✅

### P4-E1 · Domain Models (vault.dart, dedup.dart) ✅

### P4-E2 · Repository Interfaces & Fakes ✅

### P4-E3 · VaultBloc & DedupBloc ✅

### P4-E4 · HttpVaultRepository & HttpDedupRepository ✅

### P4-E5 · VaultScreen ✅

### P4-E6 · DedupReviewScreen ✅

### P4-E7 · EcosystemScreen & Navigation Wiring ✅

---

## Track P4-F — Additional Ecosystem Connectors ✅

### P4-F1 · Plugins — DLSite Connector ✅
### P4-F2 · Plugins — FANZA Connector ✅
### P4-F3 · Plugins — BookWalker Connector ✅
### P4-F4 · Plugins — Kindle Connector ✅

### P4-F5 · Services — Multi-Platform SyncService Extension ✅

Extended `SyncService` with:
- `ebook_meta_repo`, `image_meta_repo`, `video_meta_repo` fields
- `sync_discovered_items(platform, items)` method dispatching on `ResourceType`
- `parse_resource_type` helper mapping metadata strings to `ResourceType`
- Full test coverage including multi-platform, ebook, and skip-duplicate cases

---

## Track P4-G — Sync Dashboard & API ✅

### P4-G1 · Adapters — Sync API Endpoints ✅

Routes implemented:
- `POST /api/v1/ecosystem/:platform/sync` — trigger sync job
- `GET  /api/v1/ecosystem/:platform/syncs` — list sync jobs
- `GET  /api/v1/ecosystem/:platform/syncs/:id` — get job detail
- `GET  /api/v1/ecosystem/status` — all-platform overview

### P4-G2 · Frontend — Sync Dashboard Screen ✅

Implemented:
- `SyncBloc` with events: `LoadEcosystemStatus`, `LoadPlatformSyncs`, `TriggerSync`
- `HttpSyncRepository` + `SyncRepository` interface
- `sync.dart` domain models (`SyncJob`, `PlatformStatus`)
- `SyncDashboardScreen` with per-platform cards, trigger button, and history drill-down
- Wired into `main.dart` as `BlocProvider<SyncBloc>`

### P4-G3 · Frontend — Ecosystem Settings Screen ✅

Implemented:
- `EcosystemSettingsScreen` with per-platform credential management tiles
- Bottom sheet credential entry (stores to vault via `VaultBloc`)
- `StoreCredential` event added to `VaultBloc`
- `EcosystemScreen` updated with navigation to Sync Dashboard and Settings

---

## Track P4-H — Contract Sync & Verification ✅

### P4-H1 · OpenAPI / Contract Sync ✅

- sync_handler endpoints registered in `openapi.rs`
- `SyncJobResponse`, `SyncJobListResponse`, `TriggerSyncRequest`, `TriggerSyncResponse`, `EcosystemStatusResponse`, `PlatformStatus` added to component schemas

### P4-H2 · Final Verification ✅

- `cargo test --workspace`: all crates green (251 tests)
- `flutter test`: 159 tests green

### P4-H3 · Context Update ✅

Updated in this file and CONTEXT.md.

---

## Phase 5 Deferred Todo

### P5-D1 · Frontend Platform Build Setup

Deferred roadmap item:
- Add `frontend/android/`
- Add `frontend/ios/`
- Review desktop host-project setup consistency
- Document local build prerequisites and CI expectations

This is **not** Phase 4 implementation scope. It is a Phase 5 cross-cutting platform-delivery task.

### P5-D2 · CI/CD Pipeline

Deferred roadmap item:
- GitHub Actions workflows for backend + frontend
- Docker image build and push
- Automated test gating on PRs

---

## Suggested Execution Order

1. P4-A1 ✅
2. P4-B1 ✅
3. P4-B2 ✅
4. P4-B3 ✅
5. P4-B4 ✅
6. P4-B5 ✅
7. P4-C1 ✅
8. P4-C2 ✅
9. P4-C3 ✅
10. P4-C4 ✅
11. P4-D1 ✅
12. P4-D2 ✅
13. P4-D3 ✅
14. P4-D4 ✅
15. P4-D5 ✅
16. P4-D6 ✅
17. P4-E1 ✅
18. P4-E2 ✅
19. P4-E3 ✅
20. P4-E4 ✅
21. P4-E5 ✅
22. P4-E6 ✅
23. P4-E7 ✅
24. P4-F1 ✅
25. P4-F2 ✅
26. P4-F3 ✅
27. P4-F4 ✅
28. P4-F5 ✅
29. P4-G1 ✅
30. P4-G2 ✅
31. P4-G3 ✅
32. P4-H1 ✅
33. P4-H2 ✅
34. P4-H3 ✅
