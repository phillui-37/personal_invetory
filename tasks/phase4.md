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

### Sync (planned — not yet implemented)
```
POST /api/v1/sync/trigger
GET  /api/v1/sync/jobs
GET  /api/v1/sync/jobs/:id
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

## Track P4-E — Frontend Ecosystem UX 🔄

### P4-E1 · Domain Models (vault.dart, dedup.dart) ✅

Define Flutter domain models:
- Vault: `VaultStatus`, `VaultCredential`, `StoredPlatform`
- Dedup: `DedupWarning`, `DedupScanResult`, `MergeRequest`

**Acceptance:** Models compile. Unit tests for serialization pass.

### P4-E2 · Repository Interfaces & Fakes ✅

Define repository contracts and test doubles:
- `VaultRepository` interface + `FakeVaultRepository`
- `DedupRepository` interface + `FakeDedupRepository`

**Acceptance:** `flutter test` green for repository contract tests.

### P4-E3 · VaultBloc & DedupBloc ✅

Implement BLoC state management:
- `VaultBloc`: initialize, unlock, lock, store/retrieve/delete credentials
- `DedupBloc`: trigger scan, load warnings, dismiss, merge

**Acceptance:** `flutter test` green for bloc tests using fake repositories.

### P4-E4 · HttpVaultRepository & HttpDedupRepository 🔄

Implement HTTP-backed repositories:
- `HttpVaultRepository` wired to vault API endpoints
- `HttpDedupRepository` wired to dedup API endpoints

**Acceptance:** `flutter test` green. HTTP calls map to correct endpoints.

### P4-E5 · VaultScreen

Build vault management UI:
- Vault status display (locked/unlocked/uninitialized)
- Initialize flow (set master password)
- Unlock/lock actions
- Credential list per platform
- Store/delete credentials

**Acceptance:** `flutter test` green. Widget tests cover vault states and interactions.

### P4-E6 · DedupReviewScreen

Build duplicate review UI:
- Warning list with similarity scores
- Side-by-side resource comparison
- Dismiss and merge actions
- Scan trigger button

**Acceptance:** `flutter test` green. Widget tests cover warning display, dismiss, and merge.

### P4-E7 · EcosystemScreen & Navigation Wiring

Build ecosystem hub screen and wire into app navigation:
- Ecosystem overview (connected platforms, sync status)
- Navigation from main drawer/tab to ecosystem section
- Entry points to vault and dedup screens

**Acceptance:** `flutter test` green. Navigation to ecosystem, vault, and dedup screens works.

---

## Track P4-F — Additional Ecosystem Connectors

### P4-F1 · Plugins — DLSite Connector

Implement DLSite browser-session-based scraping:
- Authenticate via stored cookies (from vault)
- Scrape owned game/content library
- Map to `DiscoveredItem` format

**Acceptance:** `cargo test -p plugins` green. DLSite response parsing tested with fixture data.

### P4-F2 · Plugins — FANZA Connector

Implement FANZA browser-session-based scraping:
- Authenticate via stored cookies (from vault)
- Scrape purchased content library
- Map to `DiscoveredItem` format

**Acceptance:** `cargo test -p plugins` green. FANZA response parsing tested with fixture data.

### P4-F3 · Plugins — BookWalker Connector

Implement BookWalker ebook library sync:
- API or scraping-based library retrieval
- Map to `DiscoveredItem` format with ebook metadata

**Acceptance:** `cargo test -p plugins` green. BookWalker response parsing tested with fixture data.

### P4-F4 · Plugins — Kindle Connector

Implement Kindle ebook library sync:
- API or scraping-based library retrieval
- Map to `DiscoveredItem` format with ebook metadata

**Acceptance:** `cargo test -p plugins` green. Kindle response parsing tested with fixture data.

### P4-F5 · Services — Multi-Platform SyncService Extension

Extend `SyncService` to support all connectors:
- Connector registry keyed by platform
- Platform-specific import logic (games vs ebooks)
- Unified sync job tracking across platforms

**Acceptance:** `cargo test -p services` green. Multi-platform sync tested with mock connectors.

---

## Track P4-G — Sync Dashboard & API

### P4-G1 · Adapters — Sync API Endpoints

Add sync HTTP routes:
- `POST /api/v1/sync/trigger` (trigger sync for a platform)
- `GET /api/v1/sync/jobs` (list sync job history)
- `GET /api/v1/sync/jobs/:id` (get sync job details)

**Acceptance:** `cargo test -p adapters` green. All sync routes registered and reachable.

### P4-G2 · Frontend — Sync Dashboard Screen

Build sync management UI:
- Trigger sync per connected platform
- Sync job history list (status, item counts, timestamps)
- Sync job detail view with imported items

**Acceptance:** `flutter test` green. Widget tests cover sync trigger and history display.

### P4-G3 · Frontend — Ecosystem Settings Screen

Build per-platform configuration UI:
- Connection status per platform (connected/disconnected)
- API key / credential entry (via vault)
- Platform-specific settings (Steam API key, DLSite cookies, etc.)

**Acceptance:** `flutter test` green. Widget tests cover platform connection flow.

---

## Track P4-H — Contract Sync & Verification

### P4-H1 · OpenAPI / Contract Sync

- Refresh backend OpenAPI coverage for all Phase 4 endpoints
- Keep frontend contract usage aligned with new DTOs
- Regenerate generated client if that remains the chosen contract flow

### P4-H2 · Final Verification

Run:
- `cd backend && cargo test --workspace`
- `cd frontend && flutter test`

### P4-H3 · Context Update

Update `CONTEXT.md` with:
- Final Phase 4 scope and implementation status
- New API endpoints and architecture additions
- Explicit Phase 5 deferral for Android/iOS/desktop build setup

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
20. P4-E4 🔄
21. P4-E5
22. P4-E6
23. P4-E7
24. P4-F1
25. P4-F2
26. P4-F3
27. P4-F4
28. P4-F5
29. P4-G1
30. P4-G2
31. P4-G3
32. P4-H1
33. P4-H2
34. P4-H3
