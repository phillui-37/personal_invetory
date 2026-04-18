# Phase 3 Tasks — Resource Expansion

**Spec context:** `CONTEXT.md`, `TODO.md`, `AGENTS.md`  
**Approach:** Backend-first, vertical slices by resource type  
**Methodology:** TDD — Red → Green → Refactor on every task  
**Scope:** Add `Image`, `Video`, and `Game` resource support end to end  
**Deferred:** Android/iOS/desktop build setup belongs to **Phase 4** as cross-cutting platform work

---

## Current State

- Backend and frontend only support `Ebook` and `WebReader`
- Backend routes, services, repositories, and OpenAPI are split around those two types only
- Frontend list/detail/add flows are also hard-wired to those two types
- SQLite is the only working runtime DB adapter today
- Flutter code is portable, but repo-level Android/iOS/desktop host setup is still missing

---

## Phase 3 Objectives

1. Add `Image` resource support across backend and frontend
2. Add `Video` resource support across backend and frontend
3. Add `Game` resource support across backend and frontend
4. Refactor frontend navigation/list/detail shape so five resource types remain usable
5. Keep ecosystem integrations and platform build scaffolding out of Phase 3

---

## Resource-Type Direction

### Image

Phase 3 image support should cover:
- base resource CRUD
- metadata suitable for owned files first
- optional dimensions / format fields if cheap to support
- normal resource-location handling

### Video

Phase 3 video support should cover:
- base resource CRUD
- metadata suitable for owned files first
- optional duration / format fields if cheap to support
- normal resource-location handling

### Game

Phase 3 game support should cover:
- base resource CRUD
- manual-entry-friendly metadata first
- explicit support for platform/store/manual notes
- Nintendo/manual workflows treated as first-class manual input

Phase 3 does **not** include:
- Steam / DLSite / FANZA integration logic
- account sync
- storefront scraping/import

Those stay deferred for Phase 4.

---

## Track P3-A — Roadmap Lock

### P3-A1 · Phase 3 Requirement Lock

Map the relevant deferred requirements from `TODO.md` into this phase:
- resource types: image, video, game
- Nintendo/manual entry expectations
- mobile/desktop build setup stays deferred and moved into Phase 4 roadmap language

**Acceptance:** Phase 3 scope is explicit and Phase 4 owns platform build setup.

---

## Track P3-B — Shared Foundations

### P3-B1 · Domain Expansion

Extend backend and frontend resource enums/models:
- backend `ResourceType`: add `Image`, `Video`, `Game`
- frontend `ResourceType`: add matching values
- define metadata types for each new resource kind

**Acceptance:** Shared type systems can represent all five resource kinds without hacks.

### P3-B2 · Validation and Repository Contracts

Add pure validation and repository seams for the new types:
- backend `use_cases` validation modules
- backend repository traits / metadata ports as needed
- frontend repository contracts and in-memory repository support

**Acceptance:** New types have the same clean boundaries as ebook/web-reader.

### P3-B3 · SQLite Storage and Migrations

Add Phase 3 SQLite persistence:
- migrations for new metadata tables
- SQLite repositories for image/video/game metadata
- explicit posture for Postgres remains fail-fast until real support exists

**Acceptance:** SQLite remains the working reference adapter for all Phase 3 resource types.

---

## Track P3-C — Image Slice

### P3-C1 · Backend Image Support

Implement:
- image DTOs
- image service
- image handlers/routes
- OpenAPI registration
- tests

### P3-C2 · Frontend Image Support

Implement:
- image repository support
- image add/list/detail UI
- BLoC and widget tests

**Acceptance:** Image resources work end to end.

---

## Track P3-D — Video Slice

### P3-D1 · Backend Video Support

Implement:
- video DTOs
- video service
- video handlers/routes
- OpenAPI registration
- tests

### P3-D2 · Frontend Video Support

Implement:
- video repository support
- video add/list/detail UI
- BLoC and widget tests

**Acceptance:** Video resources work end to end.

---

## Track P3-E — Game Slice

### P3-E1 · Backend Game Support

Implement:
- game DTOs
- game service
- game handlers/routes
- OpenAPI registration
- tests

### P3-E2 · Frontend Game Support

Implement:
- game repository support
- game add/list/detail UI
- BLoC and widget tests

### P3-E3 · Manual-Entry-First Game Metadata

Ensure the game flow handles manual entry well:
- Nintendo/manual ownership paths
- platform/store/manual notes
- no ecosystem coupling yet

**Acceptance:** Game resources work end to end without depending on ecosystem integrations.

---

## Track P3-F — Frontend UX Refactor

### P3-F1 · Resource List / Navigation Refactor

Replace or refactor the current two-tab list shape so it can hold:
- Ebooks
- Web Readers
- Images
- Videos
- Games

This should stay simple and readable. Do not pile five-type branching into the current tab shape if it becomes garbage.

**Acceptance:** Resource browsing remains usable after adding three more types.

### P3-F2 · Shared Add / Detail Flow Review

Review whether add/detail screens can stay type-specific with shared helpers, or need a light structural split.

**Acceptance:** Each resource type has a clear add/detail path with minimal duplicated junk.

---

## Track P3-G — Contract Sync and Verification

### P3-G1 · OpenAPI / Contract Sync

- refresh backend OpenAPI coverage
- keep frontend contract usage aligned with new DTOs
- regenerate generated client if that remains the chosen contract flow

### P3-G2 · Final Verification

Run:
- `cd backend && cargo test --workspace`
- `cd frontend && flutter test`

### P3-G3 · Context Update

Update `CONTEXT.md` with:
- final Phase 3 scope
- chosen frontend navigation shape
- explicit Phase 4 deferral for Android/iOS/desktop build setup

---

## Phase 4 Deferred Todo

### P4-D1 · Frontend Platform Build Setup

Deferred roadmap item:
- add `frontend/android/`
- add `frontend/ios/`
- review desktop host-project setup consistency
- document local build prerequisites and CI expectations

This is **not** Phase 3 implementation scope. It is a Phase 4 cross-cutting platform-delivery task.

---

## Suggested Execution Order

1. P3-A1
2. P3-B1
3. P3-B2
4. P3-B3
5. P3-C1
6. P3-C2
7. P3-D1
8. P3-D2
9. P3-E1
10. P3-E2
11. P3-E3
12. P3-F1
13. P3-F2
14. P3-G1
15. P3-G2
16. P3-G3
