# Phase 9 Design Spec — Real API Integration + Advanced Features

**Date**: 2026-04-20
**Author**: Copilot (Claude Opus 4.6) + Phil
**Status**: Draft

---

## 1. Goal

Phase 9 delivers two tracks:

- **Track A**: Real API integration for Steam, DLSite, FANZA, and Kindle (with OTP support). Production HTTP calls with retry middleware. Fixtures only in CI.
- **Track B**: Wire existing SearchFilterBar into ResourceListScreen with shared filter state across all 5 resource tabs. Backend sort/facet query params on all list endpoints.

---

## 2. Scope

### In Scope

- HAR file parsing → extract real API response structures + cookies + endpoints
- Replace synthetic fixture data with HAR-extracted real-structure fixtures
- Real HTTP client for Steam (reqwest with API key)
- Headless Chromium integration for DLSite, FANZA, Kindle (cookie-based auth)
- OTP interaction service for Kindle (headless Chromium pauses, user enters OTP via API)
- Session cookie lifecycle (store in vault, refresh on 401/403)
- Wire SearchFilterBar widget into ResourceListScreen
- SearchFilterBloc (shared global filter state across 5 tabs)
- Backend `sort_by`, `sort_order`, `with_facets` query params on all 5 list endpoints
- SQL-level sorting (ORDER BY) for both SQLite and PostgreSQL

### Out of Scope (Phase 10+)

- BookWalker connector (no HAR data, no credentials captured)
- Tag autocomplete / typeahead
- Search history replay widget
- Mobile hardening (iOS signing, Android release APK)
- Performance profiling
- Bulk operations polish

---

## 3. Architecture

### 3.1 Track A — Real API Integration

#### Overall Sync Flow

```
User triggers sync (frontend SyncDashboardScreen)
  → SyncBloc.add(TriggerSync(platform))
  → POST /api/v1/sync/trigger { platform }
  → SyncService creates SyncJob(Running)
  → SyncService retrieves credentials from VaultService
  → SyncService calls connector.discover_items(credentials)
      → Steam: reqwest GET /IPlayerService/GetOwnedGames with API key
      → DLSite: ChromiumSession with cookies → navigate library → scrape DOM
      → FANZA: ChromiumSession with cookies → navigate library → scrape DOM
      → Kindle: ChromiumSession with login + OTP pause → scrape library
  → SyncService deduplicates (case-insensitive title)
  → SyncService creates new resources + type-specific metadata
  → SyncJob updated (Completed with counts / Failed with error)
  → Frontend: SyncBloc polls or SSE notification
```

#### Credential Format (stored in Vault as JSON bytes)

| Platform | Format |
|---|---|
| Steam | `{"api_key": "...", "steam_id": "..."}` |
| DLSite | `{"cookies": [{"name":"...","value":"...","domain":"...","path":"..."}], "username": "...", "password": "..."}` |
| FANZA | `{"cookies": [{"name":"...","value":"...","domain":"...","path":"..."}], "username": "...", "password": "..."}` |
| Kindle | `{"username": "...", "password": "...", "marketplace": "jp"}` |

#### Session Cookie Lifecycle

1. **First sync**: Headless Chromium authenticates with stored username/password → extracts session cookies → stores cookies in Vault
2. **Subsequent syncs**: Load cookies from Vault → set on Chromium → navigate library page
3. **Cookie expired (401/403)**: Re-authenticate using stored username/password → update cookies in Vault
4. **OTP required (Kindle)**: Chromium detects OTP form → backend emits SSE notification "OTP Required for Kindle" → user submits OTP via `POST /api/v1/sync/otp` → Chromium fills OTP → continues

#### New Backend Components

| Component | Crate | File | Responsibility |
|---|---|---|---|
| `HttpConnectorClient` | `plugins` | `src/http_client.rs` | reqwest wrapper with retry middleware, timeout, User-Agent |
| `ChromiumSession` | `plugins` | `src/chromium_session.rs` | Real `BrowserPage` impl using headless Chromium (via `headless_chrome` or `chromiumoxide`) |
| `OtpInteractionService` | `services` | `src/otp_service.rs` | Manages OTP pause/resume. Stores pending OTP request, exposes SSE event + submit endpoint |
| HAR parser utility | `plugins` | `src/har_parser.rs` | Parse HAR JSON → extract endpoints, cookies, response bodies |

#### Modified Backend Components

| File | Change |
|---|---|
| `plugins/src/ecosystem/steam.rs` | Replace placeholder with real Steam API URL. Use `HttpConnectorClient` for GET. Parse `GetOwnedGames` response. |
| `plugins/src/ecosystem/dlsite.rs` | Replace placeholder URLs with HAR-discovered endpoints. Use `ChromiumSession` for auth + DOM scraping. Real library page navigation. |
| `plugins/src/ecosystem/fanza.rs` | Replace placeholder URLs. Use `ChromiumSession`. Real DMM library page navigation. |
| `plugins/src/ecosystem/kindle.rs` | Implement Amazon JP login flow. OTP detection + pause. ChromiumSession for library scraping. |
| `services/src/sync_service.rs` | Wire VaultService credential retrieval before connector calls. Handle OTP flow for Kindle. |
| `adapters/src/sync_handler.rs` | Add `POST /api/v1/sync/otp` endpoint for OTP submission. |
| `adapters/src/routes.rs` | Register OTP endpoint. |
| `app/src/runtime.rs` | Wire `OtpInteractionService`, `ChromiumSession` factory. |

#### Retry Middleware Integration

The existing `retry_middleware.rs` (exponential backoff, transient/permanent classification) is used by `HttpConnectorClient`:

- Steam API: retry on 429/5xx, fail on 401 (bad API key)
- DLSite/FANZA: retry on 5xx, re-auth on 401/403 (cookie expired)
- Rate limit: respect `Retry-After` header

#### Feature Gating

- `real-plugins` cargo feature enables real HTTP/Chromium code
- Without feature: connectors return empty vec (existing behavior)
- CI runs without `real-plugins` → uses fixture data
- Production runs with `real-plugins` → real API calls

### 3.2 Track B — Search Filter Wiring + Backend Sort/Facet

#### Frontend Architecture

```
SearchFilterBar (stateless widget)
  ↕ callbacks
SearchFilterBloc (global, provided at app root)
  → emits SearchFilterState { tags, sortBy, sortOrder, filterLogic }
  → ResourceListScreen listens
  → On state change: all 5 resource BLoCs re-dispatch Load with filter params
```

**New component**: `SearchFilterBloc`
- Events: `UpdateTags(List<String>)`, `UpdateSort(SortField, SortOrder)`, `UpdateFilterLogic(FilterLogic)`, `ClearFilters`
- State: `SearchFilterState { tags, sortBy, sortOrder, filterLogic }`
- Provided at app root in `main.dart` via `BlocProvider`

**Modified components**:
- `ResourceListScreen`: Insert `SearchFilterBar` above `TabBarView`. Listen to `SearchFilterBloc` state changes. Re-dispatch load events on all 5 BLoCs.
- Each resource BLoC's Load event: Add optional `tags`, `sortBy`, `sortOrder`, `filterLogic` parameters.
- Each HTTP repository method: Pass filter/sort query params to API call.

#### Backend Architecture

**Handler query params** (all 5 list endpoints):
```
GET /api/v1/inventory/ebooks/list?tags=fiction&tags=mystery&logic=and&sort_by=title&sort_order=asc&with_facets=true
```

**New query struct**:
```rust
#[derive(Deserialize, IntoParams)]
pub struct ListQueryParams {
    pub tags: Option<Vec<String>>,
    pub logic: Option<String>,       // "and" | "or", default "or"
    pub sort_by: Option<String>,     // "title" | "date_added", default "date_added"
    pub sort_order: Option<String>,  // "asc" | "desc", default "desc"
    pub with_facets: Option<bool>,   // default false
}
```

**SQL sorting**: Applied in repository layer via dynamic ORDER BY clause.
- SQLite: `ORDER BY LOWER(title) ASC` or `ORDER BY created_at DESC`
- PostgreSQL: Same, with `LOWER()` for case-insensitive title sort

**Facets**: Computed via `COUNT(*) GROUP BY format` query. Returned in response body alongside results when `with_facets=true`.

---

## 4. Data Model Changes

### New Tables: None

No new tables required. Existing tables cover all needs:
- `sync_jobs` — tracks sync operations
- `credentials` — stores encrypted platform credentials
- `vault_config` — vault status

### Modified Responses

List endpoint responses get optional `facets` field:
```json
{
  "items": [...],
  "facets": {
    "format_counts": [
      {"format": "pdf", "count": 42},
      {"format": "epub", "count": 18}
    ]
  }
}
```

---

## 5. Error Handling

### Track A Errors

| Error | HTTP | Handling |
|---|---|---|
| Invalid API key (Steam) | 401 | Permanent → fail sync job, notify user "Invalid Steam API key" |
| Expired cookies (DLSite/FANZA) | 401/403 | Re-auth attempt → if re-auth fails, fail sync job |
| Rate limited | 429 | Retry with backoff (existing middleware) |
| Server error | 5xx | Retry up to 3 times (existing middleware) |
| OTP timeout (Kindle) | — | 5 minute timeout → fail sync job "OTP not provided in time" |
| Chromium not available | — | Fail fast with "Chromium not configured" error |
| Network unreachable | — | Transient → retry, then fail |

### Track B Errors

| Error | Handling |
|---|---|
| Invalid sort_by value | Default to `date_added` (no error) |
| Invalid sort_order value | Default to `desc` (no error) |
| Invalid logic value | Default to `or` (no error) |
| Unknown tag name | Ignore (filter returns no matches for that tag) |

---

## 6. Testing Strategy

### Track A: Real API Integration

**Fixture tests (CI, offline):**
- Update existing 49 fixture tests to use HAR-extracted response structures
- Add new fixture tests for: cookie parsing, credential deserialization, OTP state machine
- Target: 30+ new tests

**Integration tests (local, with real credentials):**
- Gated behind `#[cfg(feature = "real-plugins")]` + env var checks
- Steam: real API call with `STEAM_API_KEY` env
- DLSite/FANZA: require Chromium + valid cookies (manual local testing)
- Kindle: require Chromium + Amazon JP credentials (manual local testing)
- Gracefully skip if credentials not set

**Test execution:**
```bash
# CI (no real API calls)
cargo test -p plugins
cargo test -p services
cargo test -p adapters

# Local with real Steam
STEAM_API_KEY=xxx cargo test -p plugins --features real-plugins -- steam

# Local with Chromium
CHROMIUM_PATH=/usr/bin/chromium cargo test -p plugins --features real-plugins -- dlsite
```

### Track B: Search Filter Wiring

**Backend tests:**
- Sort query param parsing (6 tests)
- SQL ORDER BY generation for SQLite + PG (4 tests)
- Facet aggregation query (4 tests)
- Handler integration with sort/facet params (5 tests)

**Frontend tests:**
- SearchFilterBloc unit tests (8 tests: state transitions, clear, update)
- ResourceListScreen widget test with filter bar (4 tests)
- HTTP repository with query param passthrough (5 tests)

**Target: 36+ new tests for Track B**

---

## 7. Task Breakdown

### Track A: Real API Integration

| ID | Task | Dependencies | Estimated Tests |
|---|---|---|---|
| P9-A1 | HAR extraction + fixture upgrade | — | 10 |
| P9-A2 | HttpConnectorClient (reqwest + retry) | P9-A1 | 8 |
| P9-A3 | ChromiumSession (real BrowserPage impl) | — | 6 |
| P9-A4 | Steam real integration | P9-A1, P9-A2 | 8 |
| P9-A5 | DLSite real integration | P9-A1, P9-A3 | 8 |
| P9-A6 | FANZA real integration | P9-A1, P9-A3 | 8 |
| P9-A7 | OTP interaction service | P9-A3 | 6 |
| P9-A8 | Kindle real integration | P9-A3, P9-A7 | 8 |
| P9-A9 | Credential-from-vault wiring in SyncService | P9-A4..A8 | 4 |

### Track B: Advanced Search Features

| ID | Task | Dependencies | Estimated Tests |
|---|---|---|---|
| P9-B1 | Backend sort/facet query params | — | 10 |
| P9-B2 | SearchFilterBloc (frontend) | — | 8 |
| P9-B3 | Wire SearchFilterBar into ResourceListScreen | P9-B1, P9-B2 | 8 |
| P9-B4 | Update HTTP repositories with filter/sort params | P9-B1 | 5 |

### Execution Order

```
Phase: HAR + Foundation
  P9-A1 (HAR extraction)
  P9-A2 (HttpConnectorClient) — can parallel with A3
  P9-A3 (ChromiumSession) — can parallel with A2
  P9-B1 (Backend sort/facet) — can parallel with A1-A3
  P9-B2 (SearchFilterBloc) — can parallel with B1

Phase: Platform Connectors (sequential)
  P9-A4 (Steam) — simplest, validates patterns
  P9-A5 (DLSite) — first cookie-based connector
  P9-A6 (FANZA) — mirrors DLSite pattern

Phase: Kindle + Wiring
  P9-A7 (OTP service)
  P9-A8 (Kindle)
  P9-A9 (Credential-from-vault wiring)
  P9-B3 (Wire SearchFilterBar)
  P9-B4 (HTTP repo filter params)
```

---

## 8. Dependencies & Prerequisites

### Cargo Dependencies (new)

| Crate | Version | Purpose |
|---|---|---|
| `chromiumoxide` or `headless_chrome` | latest | Headless Chromium automation. Evaluate both during P9-A3; pick whichever compiles on musl/Alpine and supports cookie set/get. |
| `tokio` | existing | Async runtime (already present) |
| `reqwest` | existing | HTTP client (already in use for Steam) |

### Environment

| Variable | Required For | Default |
|---|---|---|
| `STEAM_API_KEY` | Steam sync | None (skip if absent) |
| `STEAM_ID` | Steam sync (user's Steam ID) | None |
| `CHROMIUM_PATH` | DLSite/FANZA/Kindle sync | `/usr/bin/chromium-browser` (Docker) |
| `OTP_TIMEOUT_SECS` | Kindle sync | 300 (5 minutes) |

### Files

| File | Purpose |
|---|---|
| `.credentials.local.md` | Amazon JP login info (gitignored) |
| `*.har` files | DLSite/FANZA API captures (gitignored) |
| `backend/plugins/tests/fixtures/real/` | HAR-extracted fixtures |

---

## 9. Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| HAR files don't contain library/purchase endpoints | Can't extract fixture structure | Browser automation scrapes DOM directly; fixtures use synthetic structure matching DOM output |
| DLSite/FANZA API changes frequently | Connector breaks | Feature-gated; fixture tests always pass; real tests gracefully skip |
| Kindle OTP flow too complex | Blocks Kindle sync | Falls back to CSV import path (already implemented) |
| `chromiumoxide` incompatible with musl/Alpine | Docker build fails | Use `headless_chrome` crate as fallback, or `chromium` binary compatibility layer |
| Session cookies expire during development | Can't test | Re-capture HAR; store fresh cookies in vault |

---

## 10. Success Criteria

- [ ] Steam sync works with real API key → creates Game resources
- [ ] DLSite sync works with headless Chromium → creates resources with correct types
- [ ] FANZA sync works with headless Chromium → creates resources with correct types
- [ ] Kindle sync works with Amazon JP login + OTP → creates Ebook resources
- [ ] All 49 existing fixture tests pass with HAR-extracted fixture structure
- [ ] 66+ new tests green (Track A: 66, Track B: 31)
- [ ] SearchFilterBar visible on ResourceListScreen with shared state across 5 tabs
- [ ] Backend list endpoints accept sort_by, sort_order, with_facets params
- [ ] Results sorted correctly in both SQLite and PostgreSQL
- [ ] Zero regressions on existing 568 tests
