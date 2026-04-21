# Phase 10 — Connector Hardening, Search UX Completion, and Release Readiness

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Carry only real unfinished work forward after Phase 9: finish the ecosystem connectors that still have production gaps, complete the deferred search UX, harden mobile release packaging, and polish the highest-friction performance and batch flows.

**Architecture:** Keep the existing hexagonal boundaries. Backend connector work stays in `backend/plugins` and `backend/services`; frontend search and batch UX stays in widgets, services, BLoCs, and repositories; mobile hardening stays in platform build files, docs, and CI. Treat stale Phase 9 checklist drift as historical noise, not new scope.

**Tech Stack:** Rust (`reqwest`, `chromiumoxide`, `serde`, `axum`), Flutter (`flutter_bloc`, local persistence package such as `shared_preferences` if needed), GitHub Actions, Android Gradle, Xcode/CocoaPods.

---

## Remaining-Work Audit

### Do **not** carry these forward

1. **`tasks/phase9.md` unchecked boxes** — git history and `CONTEXT.md` show Phase 9 was merged via `7a4ad14`, so the unchecked Phase 9 boxes are stale documentation, not remaining implementation.
2. **Batch update/copy/import foundations** — backend routes and the `BatchOperationsScreen` already exist from earlier phases.
3. **Search history core model/service** — `frontend/lib/models/search_history.dart` and `frontend/lib/services/search_history_service.dart` already exist; what remains is durability and UI replay.
4. **Android debug build + mobile CI baseline** — debug APK generation and the `mobile-builds.yml` workflow already exist; what remains is release identity/signing and device-grade packaging.

### Carry these into Phase 10

| Bucket | Why it is still open | Current evidence |
|---|---|---|
| Connector hardening | `bookwalker.rs`, `dlsite.rs`, `fanza.rs`, and `kindle.rs` still contain `TODO(network-inspection)` comments and placeholder assumptions in auth or library fetch flow. | `backend/plugins/src/ecosystem/*.rs` |
| Search UX completion | Search history is still in-memory only; no replay widget or facet UI is wired; tag entry is still plain text without suggestions. | `CONTEXT.md:567-572`, `frontend/lib/services/search_history_service.dart`, `frontend/lib/widgets/search_filter_bar.dart`, `frontend/lib/widgets/tag_chip_list.dart` |
| Mobile release readiness | Android release build still uses debug signing and example app ID; iOS verification exists, but device/release signing flow is still only partially documented. | `frontend/android/app/build.gradle.kts`, `docs/build-android.md`, `frontend/test/ios_build_config_test.dart` |
| Performance + batch polish | The codebase calls this out as future work, and the current batch screen is still raw text-field driven rather than a polished bulk workflow. | `CONTEXT.md:684-686`, `frontend/lib/screens/batch_operations_screen.dart` |

---

## Pre-Phase Baseline

- **Backend:** `cd backend && cargo test`
- **Frontend:** `cd frontend && flutter build apk --debug && flutter test`

The frontend baseline requires the debug APK first because `frontend/test/android_build_test.dart` validates `build/app/outputs/flutter-apk/app-debug.apk`.

---

## File Map

### New Files

| File | Responsibility |
|---|---|
| `backend/plugins/tests/ecosystem_bookwalker_test.rs` | BookWalker connector tests with captured-response fixtures and selector/auth checks |
| `backend/plugins/tests/fixtures/real/bookwalker_library.json` | Real-structure BookWalker library fixture captured from HAR/session data |
| `frontend/lib/services/search_history_storage.dart` | Durable persistence seam for `SearchHistoryService` |
| `frontend/lib/widgets/search_history_panel.dart` | Search history replay UI with remove/clear/reapply actions |
| `frontend/lib/widgets/facet_summary_bar.dart` | Format facet summary chips/counters for the list screen |
| `frontend/lib/widgets/tag_autocomplete_field.dart` | Shared tag suggestion/typeahead input used by search and detail tag entry |
| `frontend/test/services/search_history_storage_test.dart` | Persistence tests for durable history storage |
| `frontend/test/widgets/search_history_panel_test.dart` | Widget tests for replay/remove/clear flows |
| `frontend/test/widgets/facet_summary_bar_test.dart` | Widget tests for facet rendering and selection |
| `frontend/test/widgets/tag_autocomplete_field_test.dart` | Widget tests for tag suggestion and selection behavior |
| `docs/build-ios-device.md` | Concrete iOS device-signing and release-export steps |

### Modified Files

| File | Change |
|---|---|
| `backend/plugins/src/ecosystem/dlsite.rs` | Replace placeholder auth-check/library assumptions with HAR-backed flow; remove stale TODOs |
| `backend/plugins/src/ecosystem/fanza.rs` | Replace placeholder auth-check/library assumptions with HAR-backed flow; remove stale TODOs |
| `backend/plugins/src/ecosystem/kindle.rs` | Replace placeholder Amazon auth/library assumptions with verified flow or explicit fallback contract |
| `backend/plugins/src/ecosystem/bookwalker.rs` | Implement real BookWalker session/library flow instead of placeholder login assumptions |
| `backend/plugins/tests/ecosystem_dlsite_fanza_test.rs` | Harden DLSite/FANZA fixture and selector regression coverage |
| `backend/plugins/src/browser_session.rs` | Extend browser-session helpers only if connector hardening exposes missing primitives |
| `frontend/lib/services/search_history_service.dart` | Delegate persistence/load/save instead of in-memory-only behavior |
| `frontend/lib/widgets/search_filter_bar.dart` | Add facet rendering hook and tag-autocomplete entry path |
| `frontend/lib/widgets/tag_chip_list.dart` | Replace free-text tag add flow with suggestion-aware input |
| `frontend/lib/screens/resource_list_screen.dart` | Render search-history replay and facet summary alongside existing filter controls |
| `frontend/lib/main.dart` | Provide any new persistence dependency needed by search history |
| `frontend/lib/repositories/*_repository.dart` | Parse facet envelopes or expose facet-ready list result types if UI needs them |
| `frontend/android/app/build.gradle.kts` | Replace example app ID and debug-signed release config with env-driven release settings |
| `.github/workflows/mobile-builds.yml` | Add release-path validation when signing secrets/config are available |
| `docs/build-android.md` | Replace partial release guidance with exact repo-compatible signing/config steps |

---

## Track A — Ecosystem Connector Hardening

### Task P10-A1: Close DLSite and FANZA Network-Inspection Debt

**Goal:** Finish the real DLSite/FANZA connectors so the code matches the “real integration” claim instead of still carrying placeholder auth-check and endpoint TODOs.

**Files:**
- Modify: `backend/plugins/src/ecosystem/dlsite.rs`
- Modify: `backend/plugins/src/ecosystem/fanza.rs`
- Modify: `backend/plugins/tests/ecosystem_dlsite_fanza_test.rs`
- Modify: `backend/plugins/tests/fixtures/real/dlsite_library.json`
- Modify: `backend/plugins/tests/fixtures/real/fanza_library.json`

- [ ] Confirm the actual auth-check URL, library endpoint, and required headers from the captured HAR/session data.
- [ ] Replace the placeholder `navigate(login)`/`?output=json` assumptions with verified request targets and selector-based session validation.
- [ ] Remove stale `TODO(network-inspection)` comments once the code reflects the captured flow.
- [ ] Expand regression coverage for selector presence, cookie-header usage, parser structure, and non-200 responses.
- [ ] Verify with `cd backend && cargo test -p plugins ecosystem_dlsite_fanza_test -- --nocapture`

### Task P10-A2: Finish Kindle Browser Sync Hardening

**Goal:** Make the Kindle connector honest and production-safe: either fully implement the verified Amazon browser/library path or codify a clear fallback contract instead of the current half-real placeholder flow.

**Files:**
- Modify: `backend/plugins/src/ecosystem/kindle.rs`
- Modify: `backend/plugins/src/browser_session.rs` (only if a missing browser primitive blocks the real flow)
- Modify: `backend/plugins/tests/ecosystem_dlsite_fanza_test.rs` or create Kindle-specific plugin tests if coverage becomes too broad

- [ ] Verify the real Kindle library endpoint and required request headers/cookies from captured traffic or exported-library flow.
- [ ] Replace `TODO(network-inspection)` auth and library placeholders with either a verified browser sync flow or an explicit “CSV-only browser fallback” contract.
- [ ] Keep OTP behavior aligned with the existing backend OTP endpoint; do not introduce a second flow.
- [ ] Add or refresh parser/auth tests for the chosen contract.
- [ ] Verify with `cd backend && cargo test -p plugins kindle -- --nocapture`

### Task P10-A3: Add Real BookWalker Integration

**Goal:** Move BookWalker from deferred placeholder to a real connector with captured fixtures, verified selectors, and sync-ready library parsing.

**Files:**
- Modify: `backend/plugins/src/ecosystem/bookwalker.rs`
- Create: `backend/plugins/tests/ecosystem_bookwalker_test.rs`
- Create: `backend/plugins/tests/fixtures/real/bookwalker_library.json`
- Modify: `docs/har_extraction_guide.md` (only if BookWalker capture steps need to be recorded)

- [ ] Capture BookWalker HAR/session data and store a sanitized real-structure fixture.
- [ ] Replace placeholder login selectors and placeholder library URL assumptions with verified ones.
- [ ] Add parser/auth regression tests covering JSON shape, login form selectors, and non-success responses.
- [ ] Verify with `cd backend && cargo test -p plugins ecosystem_bookwalker_test -- --nocapture`

---

## Track B — Search UX Completion

### Task P10-B1: Make Search History Durable

**Goal:** Upgrade search history from in-memory-only behavior to app-restart durability without breaking the existing pure service contract.

**Files:**
- Modify: `frontend/lib/services/search_history_service.dart`
- Create: `frontend/lib/services/search_history_storage.dart`
- Modify: `frontend/lib/main.dart`
- Test: `frontend/test/services/search_history_service_test.dart`
- Create: `frontend/test/services/search_history_storage_test.dart`

- [ ] Introduce a storage seam so `SearchHistoryService` can load/save history instead of keeping everything process-local.
- [ ] Preserve the existing max-history and LIFO behavior.
- [ ] Keep serialization compatible with the existing `SearchHistory` JSON shape.
- [ ] Verify with `cd frontend && flutter test test/services/search_history_service_test.dart test/services/search_history_storage_test.dart`

### Task P10-B2: Add Search History Replay UI

**Goal:** Let users see recent searches, remove them, clear them, and replay them into the current list/search flow.

**Files:**
- Create: `frontend/lib/widgets/search_history_panel.dart`
- Modify: `frontend/lib/screens/resource_list_screen.dart`
- Modify: `frontend/lib/widgets/search_filter_bar.dart`
- Create: `frontend/test/widgets/search_history_panel_test.dart`

- [ ] Render recent searches near the existing filter controls instead of leaving history as a hidden service.
- [ ] Replaying a saved search must restore query/tags/sort/filter logic together, not only the free-text query.
- [ ] Support per-item delete and full clear behavior.
- [ ] Verify with `cd frontend && flutter test test/widgets/search_history_panel_test.dart`

### Task P10-B3: Surface Facets in the UI

**Goal:** Use the backend’s existing facet support so users can see and apply format-oriented refinements instead of only raw tag chips.

**Files:**
- Create: `frontend/lib/widgets/facet_summary_bar.dart`
- Modify: `frontend/lib/screens/resource_list_screen.dart`
- Modify: `frontend/lib/repositories/ebook_repository.dart`
- Modify: `frontend/lib/repositories/image_repository.dart`
- Modify: `frontend/lib/repositories/video_repository.dart`
- Modify: `frontend/lib/repositories/game_repository.dart`
- Modify: `frontend/lib/repositories/web_reader_repository.dart`
- Create: `frontend/test/widgets/facet_summary_bar_test.dart`

- [ ] Request and parse facet-aware list responses without breaking the plain-array response path.
- [ ] Render returned format counts as tappable facet chips or counters in the list UI.
- [ ] Keep facet state aligned with the existing `SearchFilterBloc` so refinements survive tab reloads.
- [ ] Verify with `cd frontend && flutter test test/widgets/facet_summary_bar_test.dart`

### Task P10-B4: Add Tag Autocomplete / Typeahead

**Goal:** Replace plain free-text tag entry with suggestion-aware input in both search filters and resource tag editing.

**Files:**
- Create: `frontend/lib/widgets/tag_autocomplete_field.dart`
- Modify: `frontend/lib/widgets/search_filter_bar.dart`
- Modify: `frontend/lib/widgets/tag_chip_list.dart`
- Create: `frontend/test/widgets/tag_autocomplete_field_test.dart`
- Test: `frontend/test/widgets/tag_chip_list_test.dart`

- [ ] Reuse existing tag models instead of creating a second tag shape.
- [ ] Suggest known tags while still allowing a new tag to be created explicitly.
- [ ] Keep current chip add/remove behavior working for both detail and search flows.
- [ ] Verify with `cd frontend && flutter test test/widgets/tag_autocomplete_field_test.dart test/widgets/tag_chip_list_test.dart`

---

## Track C — Mobile Release Readiness

### Task P10-C1: Harden Android Release Identity and Signing

**Goal:** Replace the example Android identity and debug-signed release build with an env-driven, documented release path.

**Files:**
- Modify: `frontend/android/app/build.gradle.kts`
- Modify: `docs/build-android.md`
- Modify: `.github/workflows/mobile-builds.yml`
- Test: `frontend/test/android_build_test.dart`

- [ ] Replace the example `applicationId` with a repo-owned configurable value.
- [ ] Load release-signing settings from ignored local files and/or CI secrets instead of using debug signing for release.
- [ ] Keep debug APK verification intact while adding a release-path guardrail for configured environments.
- [ ] Verify with `cd frontend && flutter build apk --debug && flutter test test/android_build_test.dart`

### Task P10-C2: Add iOS Device-Signing and Release-Export Runbook

**Goal:** Move iOS from “infrastructure exists” to a repeatable device/release process with concrete repo docs and CI guardrails.

**Files:**
- Create: `docs/build-ios-device.md`
- Modify: `.github/workflows/mobile-builds.yml`
- Test: `frontend/test/ios_build_config_test.dart`

- [ ] Document the exact provisioning, signing, bundle-ID, and archive/export steps needed for this project.
- [ ] Add CI checks that validate the presence and shape of any repo-tracked iOS configuration files that the runbook depends on.
- [ ] Keep secrets and signing assets out of git; document only the expected file names and environment variables.
- [ ] Verify with `cd frontend && flutter test test/ios_build_config_test.dart`

---

## Track D — Performance and Batch-Flow Polish

### Task P10-D1: Profile and Optimize Resource List Rendering

**Goal:** Replace vague “performance profiling” with a measured pass over the heaviest list/filter flows and implement only the fixes proven to matter.

**Files:**
- Modify: `frontend/lib/screens/resource_list_screen.dart`
- Modify: `frontend/lib/widgets/search_filter_bar.dart`
- Modify: `frontend/lib/blocs/search_filter/search_filter_bloc.dart`
- Modify: `docs/` performance notes only if measurements need to be recorded

- [ ] Capture baseline timings for tab switch, filter apply, and large-list rebuilds in profile mode.
- [ ] Fix only the hot spots confirmed by the measurements (for example: unnecessary rebuilds, repeated fetches, or large synchronous transforms).
- [ ] Keep the optimization local; do not redesign the whole search stack unless the profile data demands it.
- [ ] Verify with `cd frontend && flutter test`

### Task P10-D2: Polish Batch Operations UX

**Goal:** Upgrade batch operations from raw comma-separated text fields to a safer, lower-friction bulk workflow.

**Files:**
- Modify: `frontend/lib/screens/batch_operations_screen.dart`
- Modify: `frontend/lib/widgets/loading_widgets.dart`
- Test: `frontend/test/screens/batch_operations_screen_test.dart`
- Test: `frontend/test/widgets/loading_widgets_test.dart`

- [ ] Replace the most error-prone raw text-field flows with clearer affordances where the current UI is obviously brittle.
- [ ] Surface validation and progress feedback inline instead of making batch actions feel fire-and-forget.
- [ ] Preserve the existing backend API shape; this is polish on top of working endpoints, not a new protocol.
- [ ] Verify with `cd frontend && flutter test test/screens/batch_operations_screen_test.dart test/widgets/loading_widgets_test.dart`

---

## Execution Order Summary

```
Parallelizable first:
  P10-B1 (durable history)
  P10-C1 (Android release hardening)
  P10-D2 (batch UX polish)

Backend connector sequence:
  P10-A1 (DLSite/FANZA hardening)
  P10-A2 (Kindle hardening)
  P10-A3 (BookWalker real integration)

Search UX sequence:
  P10-B1 (history persistence)
    -> P10-B2 (history replay UI)
    -> P10-B3 (facet UI)
    -> P10-B4 (tag autocomplete)

Mobile sequence:
  P10-C1 (Android release hardening)
    -> P10-C2 (iOS device-signing runbook)

Polish sequence:
  P10-D1 (profile first, optimize second)
  P10-D2 (batch workflow polish)
```

---

## Phase Closure Criteria

Phase 10 is complete when all of the following are true:

1. No ecosystem connector file still carries stale `TODO(network-inspection)` debt for code paths claimed to be production-ready.
2. Search history survives app restart and can be replayed from the UI.
3. Users can see facet information and add tags through suggestions instead of raw text only.
4. Android release packaging no longer depends on debug signing or example identity values.
5. The repo has a concrete iOS device/release runbook.
6. Batch operations feel guided and validated instead of raw and brittle.
7. The Phase 9 stale checklist is explicitly treated as historical documentation, not unfinished scope.
