# Phase 6 — Progress Tracking + Tags

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Solve two core pain points from TODO.md: (1) "forgot resource progress" — persist progress (0.0–1.0) per resource, wire the existing Flutter `WebReaderProgressTracker` to save, and display/edit progress on all detail screens; (2) "no tagging" — add a tag system for grouping and filtering resources.

**Architecture:** Hexagonal — type-agnostic `resource_progress` and `tags`/`resource_tags` tables → domain traits → SQLite repos → services → axum handlers → Flutter BLoCs + UI.

**Tech Stack:** Rust/axum/rusqlite (backend), Flutter 3 / BLoC / http (frontend).

---

## Design Spec

`docs/superpowers/specs/2026-04-XX-phase6-progress-tags-design.md`

## Pre-Phase Baseline

Backend: 274 tests green. Frontend: 174 tests green.

---

## DB Schema

### Migration 0019 — `resource_progress`
```sql
CREATE TABLE IF NOT EXISTS resource_progress (
    resource_id TEXT NOT NULL,
    progress    REAL NOT NULL DEFAULT 0.0 CHECK (progress >= 0.0 AND progress <= 1.0),
    notes       TEXT,
    updated_at  TEXT NOT NULL,
    PRIMARY KEY (resource_id),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
);
```

### Migration 0020 — `tags`
```sql
CREATE TABLE IF NOT EXISTS tags (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);
```

### Migration 0021 — `resource_tags`
```sql
CREATE TABLE IF NOT EXISTS resource_tags (
    resource_id TEXT NOT NULL,
    tag_id      TEXT NOT NULL,
    PRIMARY KEY (resource_id, tag_id),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

---

## API Surface

### Progress
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/inventory/:type/:id/progress` | Get current progress (200 + body, or 404 if none recorded) |
| PATCH | `/api/v1/inventory/:type/:id/progress` | Upsert progress — body `{"progress": 0.75, "notes": "optional"}` |

### Tags
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/tags` | List all tags |
| POST | `/api/v1/tags` | Create tag — body `{"name": "sci-fi"}` |
| DELETE | `/api/v1/tags/:id` | Delete tag (cascades resource_tags) |
| GET | `/api/v1/inventory/:type/:id/tags` | Tags attached to a resource |
| POST | `/api/v1/inventory/:type/:id/tags` | Attach tag to resource — body `{"tag_id": "..."}` |
| DELETE | `/api/v1/inventory/:type/:id/tags/:tag_id` | Detach tag from resource |

### Tag filter on list endpoints
Add optional `?tag=<tag_name>` query param to all 5 existing `/:type/list` handlers — filters results to resources that have that tag.

---

## File Map

### New Files
| File | Responsibility |
|---|---|
| `backend/domain/src/progress.rs` | `ResourceProgress` struct + `ProgressRepository` trait |
| `backend/domain/src/tag.rs` | `Tag` struct + `TagRepository` trait + `ResourceTagRepository` trait |
| `backend/infrastructure/migrations/0019_create_resource_progress.sql` | progress table |
| `backend/infrastructure/migrations/0020_create_tags.sql` | tags table |
| `backend/infrastructure/migrations/0021_create_resource_tags.sql` | resource_tags junction |
| `backend/infrastructure/src/sqlite/progress.rs` | `SqliteProgressRepository` |
| `backend/infrastructure/src/sqlite/tag.rs` | `SqliteTagRepository` |
| `backend/services/src/progress.rs` | `ProgressService` (get/upsert) |
| `backend/services/src/tag.rs` | `TagService` (CRUD + resource-tag ops) |
| `backend/adapters/src/progress_handler.rs` | GET + PATCH progress handlers + DTOs |
| `backend/adapters/src/tag_handler.rs` | Tag CRUD + resource-tag handlers + DTOs |
| `frontend/lib/models/progress.dart` | `ResourceProgress` Dart model |
| `frontend/lib/models/tag.dart` | `Tag` Dart model |
| `frontend/lib/repositories/progress_repository.dart` | abstract `ProgressRepository` |
| `frontend/lib/repositories/http_progress_repository.dart` | `HttpProgressRepository` |
| `frontend/lib/repositories/tag_repository.dart` | abstract `TagRepository` |
| `frontend/lib/repositories/http_tag_repository.dart` | `HttpTagRepository` |
| `frontend/lib/blocs/progress/progress_bloc.dart` | `ProgressBloc` events/states/handler |
| `frontend/lib/blocs/tag/tag_bloc.dart` | `TagBloc` events/states/handler |
| `frontend/lib/widgets/progress_editor.dart` | Slider + save button widget |
| `frontend/lib/widgets/tag_chip_list.dart` | Tag chips with add/remove |
| `frontend/test/blocs/progress/progress_bloc_test.dart` | ProgressBloc unit tests |
| `frontend/test/blocs/tag/tag_bloc_test.dart` | TagBloc unit tests |

### Modified Files
| File | Change |
|---|---|
| `backend/domain/src/lib.rs` | `pub mod progress; pub mod tag;` |
| `backend/infrastructure/src/sqlite/mod.rs` | `pub mod progress; pub mod tag;` + SQLITE_MIGRATIONS to `[&str; 21]` |
| `backend/infrastructure/src/lib.rs` | re-export `SqliteProgressRepository`, `SqliteTagRepository` |
| `backend/infrastructure/src/factory.rs` | `AdapterBundle.progress_repo` + `.tag_repo` fields |
| `backend/services/src/lib.rs` | `mod progress; mod tag; pub use ...` |
| `backend/adapters/src/lib.rs` | `mod progress_handler; mod tag_handler; pub use ...` |
| `backend/adapters/src/state.rs` | `progress_service: Option<Arc<ProgressService>>`, `tag_service: Option<Arc<TagService>>`, `with_*` builders |
| `backend/adapters/src/routes.rs` | Progress routes + tag routes; `?tag=` filter on list routes |
| `backend/adapters/src/openapi.rs` | Progress + tag paths/schemas |
| `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs` | Progress + tag repo tests |
| `backend/services/tests/services_tdd.rs` | ProgressService + TagService tests |
| `backend/adapters/tests/handlers_tdd.rs` | Progress + tag handler tests |
| `frontend/lib/repositories/in_memory_repositories.dart` | `InMemoryProgressRepository`, `InMemoryTagRepository` |
| `frontend/lib/screens/resource_detail_screen.dart` | Add `ProgressEditor` + `TagChipList` sections |
| `frontend/lib/screens/web_reader_screen.dart` | Wire `WebReaderProgressTracker` → `ProgressBloc.UpdateProgress` |
| `frontend/lib/screens/ebook_list_screen.dart` (and other list screens) | Add tag filter UI (chip selector) |
| `frontend/lib/main.dart` | `ProgressBloc` + `TagBloc` providers + HTTP repo wiring |
| `frontend/test/support/fake_repositories.dart` | `FakeProgressRepository`, `FakeTagRepository` |

---

## Task P6-A: Domain — ResourceProgress struct + ProgressRepository trait

**Files:**
- Create: `backend/domain/src/progress.rs`
- Modify: `backend/domain/src/lib.rs`

- [ ] **Step 1: Write failing test** — in `backend/domain/src/lib.rs` tests block, add a compile-time test that `ProgressRepository` exists and `ResourceProgress` has `resource_id`, `progress`, `notes`, `updated_at` fields.

- [ ] **Step 2: Create `backend/domain/src/progress.rs`**

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::DomainError;

#[derive(Debug, Clone)]
pub struct ResourceProgress {
    pub resource_id: String,
    pub progress: f64,        // 0.0 – 1.0
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait ProgressRepository: Send + Sync {
    /// Returns None if no progress row exists for this resource_id.
    async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError>;

    /// Upsert — inserts or replaces the progress row.
    async fn upsert(
        &self,
        resource_id: &str,
        progress: f64,
        notes: Option<&str>,
    ) -> Result<ResourceProgress, DomainError>;
}
```

- [ ] **Step 3: Add `pub mod progress;` to `backend/domain/src/lib.rs`**

- [ ] **Step 4: `cargo test -p domain` — green**

---

## Task P6-B: Domain — Tag + TagRepository + ResourceTagRepository traits

**Files:**
- Create: `backend/domain/src/tag.rs`
- Modify: `backend/domain/src/lib.rs`

- [ ] **Step 1: Write failing test** — compile-time presence check for `Tag`, `TagRepository`, `ResourceTagRepository` structs/traits.

- [ ] **Step 2: Create `backend/domain/src/tag.rs`**

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::DomainError;

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Tag>, DomainError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Tag>, DomainError>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Tag>, DomainError>;
    async fn create(&self, name: &str) -> Result<Tag, DomainError>;
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
}

#[async_trait]
pub trait ResourceTagRepository: Send + Sync {
    async fn tags_for_resource(&self, resource_id: &str) -> Result<Vec<Tag>, DomainError>;
    async fn attach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError>;
    async fn detach(&self, resource_id: &str, tag_id: &str) -> Result<(), DomainError>;
    /// Returns resource_ids that have all specified tags.
    async fn resource_ids_with_tag(&self, tag_name: &str) -> Result<Vec<String>, DomainError>;
}
```

- [ ] **Step 3: Add `pub mod tag;` to `backend/domain/src/lib.rs`**

- [ ] **Step 4: `cargo test -p domain` — green**

---

## Task P6-C: Infrastructure — Migrations 0019/0020/0021 + SqliteProgressRepository

**Files:**
- Create: `backend/infrastructure/migrations/0019_create_resource_progress.sql`
- Create: `backend/infrastructure/migrations/0020_create_tags.sql`
- Create: `backend/infrastructure/migrations/0021_create_resource_tags.sql`
- Create: `backend/infrastructure/src/sqlite/progress.rs`
- Modify: `backend/infrastructure/src/sqlite/mod.rs` (add module + expand SQLITE_MIGRATIONS to `[&str; 21]`)
- Modify: `backend/infrastructure/src/lib.rs`

- [ ] **Step 1: Write failing tests** in `sqlite_repository_adapters_tdd.rs`:
  - `sqlite_progress_repository_get_returns_none_when_no_row`
  - `sqlite_progress_repository_upsert_creates_and_updates`

- [ ] **Step 2: Create the 3 migration SQL files** (schema above)

- [ ] **Step 3: Create `SqliteProgressRepository`** — implement `ProgressRepository`; upsert uses `INSERT OR REPLACE`; parse `updated_at` as RFC3339.

- [ ] **Step 4: Wire in `mod.rs`** — add `pub mod progress;`, expand `SQLITE_MIGRATIONS` array to 21 elements.

- [ ] **Step 5: `cargo test -p infrastructure` — green**

---

## Task P6-D: Infrastructure — SqliteTagRepository

**Files:**
- Create: `backend/infrastructure/src/sqlite/tag.rs`
- Modify: `backend/infrastructure/src/sqlite/mod.rs` (`pub mod tag;`)
- Modify: `backend/infrastructure/src/lib.rs`

- [ ] **Step 1: Write failing tests**:
  - `sqlite_tag_repository_create_and_list`
  - `sqlite_tag_repository_delete_cascades_resource_tags`
  - `sqlite_resource_tag_repository_attach_detach_and_filter`

- [ ] **Step 2: Create `SqliteTagRepository`** — implements both `TagRepository` and `ResourceTagRepository`; `create` uses `INSERT OR IGNORE` then fetch; `delete` relies on ON DELETE CASCADE; `resource_ids_with_tag` queries via JOIN.

- [ ] **Step 3: `cargo test -p infrastructure` — green**

---

## Task P6-E: Services — ProgressService

**Files:**
- Create: `backend/services/src/progress.rs`
- Modify: `backend/services/src/lib.rs`

- [ ] **Step 1: Write failing tests** in `services_tdd.rs`:
  - `progress_service_get_returns_none_when_not_set`
  - `progress_service_upsert_validates_range` (progress > 1.0 or < 0.0 → ValidationError)
  - `progress_service_upsert_round_trips`

- [ ] **Step 2: Create `ProgressService`**:

```rust
pub struct ProgressService {
    repo: Arc<dyn ProgressRepository>,
}
impl ProgressService {
    pub fn new(repo: Arc<dyn ProgressRepository>) -> Self { Self { repo } }
    pub async fn get(&self, resource_id: &str) -> Result<Option<ResourceProgress>, DomainError> { ... }
    pub async fn upsert(&self, resource_id: &str, progress: f64, notes: Option<&str>)
        -> Result<ResourceProgress, DomainError>
    {
        if !(0.0..=1.0).contains(&progress) {
            return Err(DomainError::ValidationError("progress must be 0.0–1.0".into()));
        }
        self.repo.upsert(resource_id, progress, notes).await
    }
}
```

- [ ] **Step 3: `cargo test -p services` — green**

---

## Task P6-F: Services — TagService

**Files:**
- Create: `backend/services/src/tag.rs`
- Modify: `backend/services/src/lib.rs`

- [ ] **Step 1: Write failing tests**:
  - `tag_service_create_normalises_name` (trim + lowercase)
  - `tag_service_create_rejects_empty_name`
  - `tag_service_attach_tag_to_resource`
  - `tag_service_detach_tag_from_resource`
  - `tag_service_tags_for_resource`
  - `tag_service_filter_resource_ids_by_tag`

- [ ] **Step 2: Create `TagService`** — normalise name (trim + lowercase) before create; delegate all ops to `TagRepository` / `ResourceTagRepository`.

- [ ] **Step 3: `cargo test -p services` — green**

---

## Task P6-G: Adapters — Progress Handlers

**Files:**
- Create: `backend/adapters/src/progress_handler.rs`
- Modify: `backend/adapters/src/lib.rs`
- Modify: `backend/adapters/src/state.rs` (add `progress_service`, `with_progress_service()`)
- Modify: `backend/adapters/src/routes.rs`
- Modify: `backend/adapters/src/openapi.rs`

- [ ] **Step 1: Write failing handler tests** in `handlers_tdd.rs`:
  - `handler_get_progress_returns_404_when_none`
  - `handler_get_progress_returns_200_when_set`
  - `handler_patch_progress_creates_and_returns_200`
  - `handler_patch_progress_rejects_out_of_range`

- [ ] **Step 2: Create `progress_handler.rs`**:
  - `get_progress(Path<(String, Uuid)>, State<AppState>) → impl IntoResponse`
  - `patch_progress(Path<(String, Uuid)>, State<AppState>, Json<PatchProgressRequest>) → impl IntoResponse`
  - `PatchProgressRequest { progress: f64, notes: Option<String> }`
  - `ProgressResponse { resource_id, progress, notes, updated_at }`

- [ ] **Step 3: Wire routes** — add `GET /api/v1/inventory/:type/:id/progress` and `PATCH /api/v1/inventory/:type/:id/progress`

- [ ] **Step 4: `cargo test -p adapters` — green**

---

## Task P6-H: Adapters — Tag Handlers

**Files:**
- Create: `backend/adapters/src/tag_handler.rs`
- Modify: `backend/adapters/src/lib.rs`
- Modify: `backend/adapters/src/state.rs` (add `tag_service`, `with_tag_service()`)
- Modify: `backend/adapters/src/routes.rs`
- Modify: `backend/adapters/src/openapi.rs`

- [ ] **Step 1: Write failing handler tests**:
  - `handler_list_tags_returns_empty_list`
  - `handler_create_tag_returns_201`
  - `handler_delete_tag_returns_204`
  - `handler_get_resource_tags_returns_list`
  - `handler_attach_tag_to_resource_returns_200`
  - `handler_detach_tag_from_resource_returns_204`

- [ ] **Step 2: Create `tag_handler.rs`**:
  - `list_tags`, `create_tag`, `delete_tag`
  - `list_resource_tags`, `attach_tag`, `detach_tag`
  - DTOs: `CreateTagRequest { name }`, `AttachTagRequest { tag_id }`, `TagResponse { id, name, created_at }`

- [ ] **Step 3: Wire routes** (see API Surface table above)

- [ ] **Step 4: `cargo test -p adapters` — green**

---

## Task P6-I: Adapters — Tag filter on list endpoints

**Files:**
- Modify: all 5 list handlers (ebook, web_reader, image, video, game)
- Modify: their respective service `list` methods to accept optional `tag_filter: Option<&str>`

- [ ] **Step 1: Write failing tests** for each list handler — `?tag=sci-fi` filters results, `?tag=` (empty) ignored.

- [ ] **Step 2: Add `tag_filter: Option<String>` to each list query struct**; propagate to service → repo (`TagService::resource_ids_with_tag` → filter).

- [ ] **Step 3: `cargo test -p adapters && cargo test -p services` — green**

---

## Task P6-J: App — Wire ProgressService + TagService in runtime

**Files:**
- Modify: `backend/infrastructure/src/factory.rs` (add `progress_repo`, `tag_repo` to `AdapterBundle`)
- Modify: `backend/app/src/runtime.rs` (construct + wire services)

- [ ] **Step 1: Update `AdapterBundle`** with `progress_repo: Arc<SqliteProgressRepository>` and `tag_repo: Arc<SqliteTagRepository>`.

- [ ] **Step 2: In `runtime.rs`**, construct `ProgressService` and `TagService`, wire into `AppState` via `with_progress_service()` / `with_tag_service()`.

- [ ] **Step 3: `cargo build -p app` — clean**

- [ ] **Step 4: `cargo test` (workspace) — all green**

---

## Task P6-K: Flutter — `ResourceProgress` + `Tag` Dart models

**Files:**
- Create: `frontend/lib/models/progress.dart`
- Create: `frontend/lib/models/tag.dart`

- [ ] **Step 1: Write failing unit test** (`flutter test`) verifying `ResourceProgress.fromJson` round-trips and `Tag.fromJson` round-trips.

- [ ] **Step 2: Create models** (Equatable, fromJson, toJson):

```dart
// progress.dart
class ResourceProgress extends Equatable {
  final String resourceId;
  final double progress;       // 0.0–1.0
  final String? notes;
  final DateTime updatedAt;
  ...
}

// tag.dart
class Tag extends Equatable {
  final String id;
  final String name;
  final DateTime createdAt;
  ...
}
```

- [ ] **Step 3: `flutter test` — green**

---

## Task P6-L: Flutter — ProgressRepository + HttpProgressRepository

**Files:**
- Create: `frontend/lib/repositories/progress_repository.dart`
- Create: `frontend/lib/repositories/http_progress_repository.dart`
- Modify: `frontend/lib/repositories/in_memory_repositories.dart` (add `InMemoryProgressRepository`)
- Modify: `frontend/test/support/fake_repositories.dart` (add `FakeProgressRepository`)

- [ ] **Step 1: Write failing tests** for `InMemoryProgressRepository.get` (returns null) and `.upsert` (stores and returns).

- [ ] **Step 2: Define `abstract class ProgressRepository`**:
  - `Future<ResourceProgress?> get(String resourceId)`
  - `Future<ResourceProgress> upsert(String resourceId, double progress, {String? notes})`

- [ ] **Step 3: Implement `HttpProgressRepository`** — maps to GET/PATCH endpoints.

- [ ] **Step 4: Implement `InMemoryProgressRepository`** + `FakeProgressRepository`.

- [ ] **Step 5: `flutter test` — green**

---

## Task P6-M: Flutter — TagRepository + HttpTagRepository

**Files:**
- Create: `frontend/lib/repositories/tag_repository.dart`
- Create: `frontend/lib/repositories/http_tag_repository.dart`
- Modify: `frontend/lib/repositories/in_memory_repositories.dart`
- Modify: `frontend/test/support/fake_repositories.dart`

- [ ] **Step 1: Write failing tests** for `InMemoryTagRepository` CRUD and `tags_for_resource`.

- [ ] **Step 2: Define `abstract class TagRepository`**:
  - `Future<List<Tag>> listAll()`
  - `Future<Tag> create(String name)`
  - `Future<void> delete(String id)`
  - `Future<List<Tag>> tagsForResource(String resourceId)`
  - `Future<void> attachTag(String resourceId, String tagId)`
  - `Future<void> detachTag(String resourceId, String tagId)`

- [ ] **Step 3: Implement `HttpTagRepository`** + `InMemoryTagRepository` + `FakeTagRepository`.

- [ ] **Step 4: `flutter test` — green**

---

## Task P6-N: Flutter — ProgressBloc + wire WebReader progress

**Files:**
- Create: `frontend/lib/blocs/progress/progress_bloc.dart`
- Create: `frontend/test/blocs/progress/progress_bloc_test.dart`
- Modify: `frontend/lib/screens/web_reader_screen.dart` (or wherever `WebReaderProgressTracker` is used)

- [ ] **Step 1: Write failing ProgressBloc tests**:
  - `ProgressBloc initial state is ProgressInitial`
  - `LoadProgress emits ProgressLoaded when repo returns value`
  - `LoadProgress emits ProgressLoaded(null) when no progress set`
  - `UpdateProgress emits ProgressUpdated then ProgressLoaded`

Events: `LoadProgress(resourceId)`, `UpdateProgress(resourceId, progress, notes?)`
States: `ProgressInitial`, `ProgressLoading`, `ProgressLoaded(ResourceProgress?)`, `ProgressUpdated`, `ProgressError(message)`

- [ ] **Step 2: Implement `ProgressBloc`**

- [ ] **Step 3: Wire `WebReaderProgressTracker.onProgressUpdate`** to call `ProgressBloc.add(UpdateProgress(...))`.

- [ ] **Step 4: `flutter test` — green**

---

## Task P6-O: Flutter — TagBloc

**Files:**
- Create: `frontend/lib/blocs/tag/tag_bloc.dart`
- Create: `frontend/test/blocs/tag/tag_bloc_test.dart`

- [ ] **Step 1: Write failing TagBloc tests**:
  - `TagBloc initial state is TagInitial`
  - `LoadAllTags emits AllTagsLoaded with tag list`
  - `LoadResourceTags emits ResourceTagsLoaded`
  - `CreateTag emits TagCreated then AllTagsLoaded`
  - `AttachTag emits TagAttached then ResourceTagsLoaded`
  - `DetachTag emits TagDetached then ResourceTagsLoaded`

Events: `LoadAllTags`, `LoadResourceTags(resourceId)`, `CreateTag(name)`, `DeleteTag(id)`, `AttachTag(resourceId, tagId)`, `DetachTag(resourceId, tagId)`
States: `TagInitial`, `TagLoading`, `AllTagsLoaded(List<Tag>)`, `ResourceTagsLoaded(List<Tag>)`, `TagCreated(Tag)`, `TagAttached`, `TagDetached`, `TagError(message)`

- [ ] **Step 2: Implement `TagBloc`**

- [ ] **Step 3: `flutter test` — green**

---

## Task P6-P: Flutter — UI Widgets + Tag Filter in List Screens

**Files:**
- Create: `frontend/lib/widgets/progress_editor.dart` (slider + percentage text + save)
- Create: `frontend/lib/widgets/tag_chip_list.dart` (chips + add via autocomplete + remove)
- Modify: `frontend/lib/screens/resource_detail_screen.dart` (add ProgressEditor + TagChipList sections)
- Modify each list screen (ebook, web_reader, image, video, game): add tag filter chip bar at top
- Modify: `frontend/lib/main.dart` (provide `ProgressBloc`, `TagBloc`, wire HTTP repos)

- [ ] **Step 1: Write failing widget tests**:
  - `ProgressEditor displays current progress from ProgressLoaded state`
  - `ProgressEditor save button dispatches UpdateProgress`
  - `TagChipList renders tags and delete button dispatches DetachTag`
  - `TagChipList add button dispatches AttachTag`

- [ ] **Step 2: Implement `ProgressEditor`** — `Slider` widget bound to `ProgressBloc` state; "Save" sends `UpdateProgress`.

- [ ] **Step 3: Implement `TagChipList`** — `Wrap` of `Chip` widgets with `onDeleted` dispatching `DetachTag`; `+` icon opens `Autocomplete<Tag>` pulling from `AllTagsLoaded`.

- [ ] **Step 4: Add `ProgressEditor` + `TagChipList` to `resource_detail_screen.dart`** inside the detail card body.

- [ ] **Step 5: Add tag filter bar** to each list screen — a horizontal scroll of `FilterChip` widgets; tapping a chip re-runs list query with `?tag=...`.

- [ ] **Step 6: `flutter test` — all green**

---

## Post-Phase

- [ ] Run full backend test suite: `cd backend && cargo test`
- [ ] Run full Flutter test suite: `cd frontend && flutter test`
- [ ] Update `CONTEXT.md` with Phase 6 completion entry
- [ ] Commit on feature branch; merge to `main`
