# Phase 10 — Connector Hardening, Search UX Completion, Release Readiness, and Repo Task Tooling

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Carry only real unfinished work forward after Phase 9 while also complementing repo-wide documentation and regression coverage: add repo-local task entrypoints, finish the ecosystem connectors that still have production gaps, complete the deferred search UX, harden mobile release packaging, and backfill the docs/tests that the next phase depends on.

**Architecture:** Keep the existing hexagonal boundaries. Backend connector work stays in `backend/plugins` and `backend/services`; frontend search and batch UX stays in widgets, services, BLoCs, and repositories; mobile hardening stays in platform build files, docs, and CI; repo task tooling stays in `bin/` plus the docs that point at it. Treat stale Phase 9 checklist drift as historical noise, not new scope.

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
| Connector hardening | `bookwalker.rs`, `dlsite.rs`, `fanza.rs`, and `kindle.rs` still contain `TODO(network-inspection)` comments and placeholder assumptions in auth or library fetch flow. Closing those gaps is not repo-only work; it still depends on sanitized capture/session input outside git. | `backend/plugins/src/ecosystem/*.rs` |
| Repo-wide docs + tests complement | Important verification rules and operator knowledge are still scattered across task docs, test files, and `CONTEXT.md`; some already-shipped flows still rely on narrow happy-path tests only. | `frontend/test/android_build_test.dart`, `CONTEXT.md`, `tasks/*.md` |
| Search UX completion | Search history is still in-memory only; no replay widget or facet UI is wired; tag entry is still plain text without suggestions. | `CONTEXT.md:567-572`, `frontend/lib/services/search_history_service.dart`, `frontend/lib/widgets/search_filter_bar.dart`, `frontend/lib/widgets/tag_chip_list.dart` |
| Mobile release readiness | Android release build still uses debug signing and example app ID; iOS verification exists, but device/release signing flow is still only partially documented. Repo changes are local, but honest release validation still depends on ignored signing config and CI secrets. | `frontend/android/app/build.gradle.kts`, `docs/build-android.md`, `frontend/test/ios_build_config_test.dart` |
| Performance + batch polish | The codebase calls this out as future work, and the current batch screen is still raw text-field driven rather than a polished bulk workflow. | `CONTEXT.md:684-686`, `frontend/lib/screens/batch_operations_screen.dart` |
| Repo task tooling | Common backend/frontend run, build, test, clean, and API-generation commands are still scattered across docs and platform-specific knowledge; there is no checked-in cross-OS `bin/` entrypoint layer or `dist/` copy flow for common repo tasks. | `README.md`, `docs/testing-matrix.md`, `docs/build-android.md`, `frontend/scripts/gen-api-client.sh`, missing `bin/` directory |

---

## Pre-Phase Baseline

- **Backend:** `cd backend && cargo test`
- **Frontend:** `cd frontend && flutter build apk --debug && flutter test`

The frontend baseline requires the debug APK first because `frontend/test/android_build_test.dart` validates `build/app/outputs/flutter-apk/app-debug.apk`.

---

## Readiness Labels

- **Repo-local executable** — the work can be implemented and meaningfully verified from the checked-out repo on this host without waiting on outside artifacts.
- **External-input-dependent** — the code/doc work lives here, but closing the task honestly still depends on fresh or sufficient sanitized HAR/session/input artifacts that are not committed.
- **Secret/config-dependent** — the repo changes are local, but release-path validation only counts when ignored local config files or CI secrets exist.
- **Host-limited validation** — the code can land from this host, but one smoke step still needs the target host or shell before it can be claimed fully validated.

---

## Cross-Cutting Documentation and Verification Rules

### Documentation Rule

Every Phase 10 task must leave behind repo-facing documentation, not just code:

1. **Connector tasks** update `docs/har_extraction_guide.md` with the verified endpoint, selector, header, cookie, and sanitization rules they depend on.
2. **Search and batch UX tasks** append the shipped behavior, user-facing constraints, and state rules to `CONTEXT.md`.
3. **Mobile tasks** update the concrete build guides (`docs/build-android.md`, `docs/build-ios-device.md`) and keep CI expectations aligned with those guides.
4. **Performance tasks** record baseline numbers, what changed, and post-fix measurements in `CONTEXT.md` so later phases do not repeat the same profiling work.

### Verification Rule

Every Phase 10 task needs both **narrow tests** and **broader regression coverage**:

1. **Backend connector work** must pass the targeted connector test file **and** the full plugin suite: `cd backend && cargo test -p plugins`.
2. **Frontend search, mobile, and batch work** must pass targeted tests **and** the broader Flutter suite: `cd frontend && flutter build apk --debug && flutter test`.
3. No task is done when only the new test passes but the surrounding package or screen regressions are unverified.

---

## File Map

### New Files

| File | Responsibility |
|---|---|
| `backend/plugins/tests/ecosystem_bookwalker_test.rs` | BookWalker connector tests with captured-response fixtures and selector/auth checks |
| `backend/plugins/tests/ecosystem_kindle_test.rs` | Kindle-specific auth/library regression tests instead of hiding Kindle coverage in unrelated plugin files |
| `backend/plugins/tests/fixtures/real/bookwalker_library.json` | Real-structure BookWalker library fixture captured from HAR/session data |
| `docs/testing-matrix.md` | Repo-wide test commands, prerequisites, and suite ownership for already-shipped and Phase 10 work |
| `bin/app` | POSIX repo-entry dispatcher for repo start/build/test/clean/gen-api flows |
| `bin/app.ps1` | Windows PowerShell repo-entry dispatcher for the same cross-OS contract |
| `frontend/lib/services/search_history_storage.dart` | Durable persistence seam for `SearchHistoryService` |
| `frontend/lib/widgets/search_history_panel.dart` | Search history replay UI with remove/clear/reapply actions |
| `frontend/lib/widgets/facet_summary_bar.dart` | Format facet summary chips/counters for the list screen |
| `frontend/lib/widgets/tag_autocomplete_field.dart` | Shared tag suggestion/typeahead input used by search and detail tag entry |
| `frontend/test/services/search_history_storage_test.dart` | Persistence tests for durable history storage |
| `frontend/test/widgets/search_history_panel_test.dart` | Widget tests for replay/remove/clear flows |
| `frontend/test/widgets/facet_summary_bar_test.dart` | Widget tests for facet rendering and selection |
| `frontend/test/widgets/tag_autocomplete_field_test.dart` | Widget tests for tag suggestion and selection behavior |
| `frontend/test/screens/resource_list_screen_search_history_test.dart` | Resource-list integration tests for history replay and persisted filter restoration |
| `frontend/test/screens/resource_list_screen_facets_test.dart` | Resource-list integration tests for facet rendering and selection |
| `frontend/test/screens/batch_operations_validation_test.dart` | Batch-flow validation/progress tests beyond the existing happy-path screen checks |
| `docs/build-ios-device.md` | Concrete iOS device-signing and release-export steps |
| `bin/app` | POSIX task dispatcher for macOS and Arch Linux repo workflows |
| `bin/app.ps1` | PowerShell task dispatcher for Windows repo workflows |
| `docs/superpowers/specs/2026-04-22-bin-dispatcher-design.md` | Approved design for the cross-OS bin dispatcher and `dist/` output contract |

### Modified Files

| File | Change |
|---|---|
| `backend/plugins/src/ecosystem/dlsite.rs` | Replace placeholder auth-check/library assumptions with HAR-backed flow; remove stale TODOs |
| `backend/plugins/src/ecosystem/fanza.rs` | Replace placeholder auth-check/library assumptions with HAR-backed flow; remove stale TODOs |
| `backend/plugins/src/ecosystem/kindle.rs` | Replace placeholder Amazon auth/library assumptions with verified flow or explicit fallback contract |
| `backend/plugins/src/ecosystem/bookwalker.rs` | Implement real BookWalker session/library flow instead of placeholder login assumptions |
| `backend/plugins/tests/ecosystem_dlsite_fanza_test.rs` | Harden DLSite/FANZA fixture and selector regression coverage |
| `docs/har_extraction_guide.md` | Keep connector capture, sanitization, endpoint, and selector guidance in sync with shipped code |
| `backend/services/tests/services_tdd.rs` | Strengthen service-level regression coverage for already-shipped sync/search flows that Phase 10 extends |
| `backend/plugins/src/browser_session.rs` | Extend browser-session helpers only if connector hardening exposes missing primitives |
| `README.md` | Reflect the already-landed current-focus wording that now includes repo task tooling alongside the active Phase 10 tracks |
| `frontend/lib/services/search_history_service.dart` | Delegate persistence/load/save instead of in-memory-only behavior |
| `frontend/lib/widgets/search_filter_bar.dart` | Add facet rendering hook and tag-autocomplete entry path |
| `frontend/lib/widgets/tag_chip_list.dart` | Replace free-text tag add flow with suggestion-aware input |
| `frontend/lib/screens/resource_list_screen.dart` | Render search-history replay and facet summary alongside existing filter controls |
| `frontend/test/screens/resource_list_screen_test.dart` | Backfill regression coverage for already-shipped list/filter flows while Phase 10 extends the screen |
| `frontend/test/screens/batch_operations_screen_test.dart` | Backfill validation/progress regressions in the existing batch screen |
| `frontend/lib/main.dart` | Provide any new persistence dependency needed by search history |
| `frontend/lib/repositories/*_repository.dart` | Parse facet envelopes or expose facet-ready list result types if UI needs them |
| `frontend/android/app/build.gradle.kts` | Replace example app ID and debug-signed release config with env-driven release settings |
| `.github/workflows/mobile-builds.yml` | Add release-path validation when signing secrets/config are available |
| `docs/build-android.md` | Replace partial release guidance with exact repo-compatible signing/config steps |
| `CONTEXT.md` | Record shipped UX rules, profiling results, and final Phase 10 behavior summaries |
| `README.md` | Point contributors at the new `bin/` entrypoints once they exist |

---

## Track 0 — Overall Documentation and Regression Complement

### Task P10-A0: Backfill Repo-Wide Docs and Regression Coverage (✅ Already complete at base SHA)

**Goal:** Complement the overall project docs and tests so the repo stops depending on tribal knowledge and narrow happy-path checks for already-shipped features.

> **Status:** Already complete at base SHA `5a7f91091158c37b5130244d29af1310fb0379c1`. Keep this entry as closed history only. Do **not** schedule, reopen, or rerun it as part of the first execution wave; create a new follow-up task instead if docs/tests regress later.

**Readiness:** **Already completed at base SHA.** Historical record only; not executable backlog for the next wave.

**Files:**
- Create: `docs/testing-matrix.md`
- Modify: `CONTEXT.md`
- Modify: `docs/build-android.md`
- Modify: `docs/har_extraction_guide.md`
- Modify: `backend/services/tests/services_tdd.rs`
- Modify: `frontend/test/screens/resource_list_screen_test.dart`
- Modify: `frontend/test/screens/batch_operations_screen_test.dart`

- [x] Build a shipped-feature matrix covering backend workspace tests, Flutter test prerequisites, ecosystem capture docs, search/filter flows, and batch workflows.
- [x] Write `docs/testing-matrix.md` with exact commands, prerequisites, and suite ownership, including the `flutter build apk --debug` prerequisite for the Flutter suite.
- [x] Strengthen existing service-level and screen-level regression tests for already-shipped flows that Phase 10 will extend.
- [x] Append the repo-wide documentation/testing complement summary to `CONTEXT.md`.
- [x] Verify with `cd backend && cargo test && cd ../frontend && flutter build apk --debug && flutter test`

---

## Track A — Ecosystem Connector Hardening

### Task P10-A1: Close DLSite and FANZA Network-Inspection Debt

**Goal:** Finish the real DLSite/FANZA connectors so the code matches the “real integration” claim instead of still carrying placeholder auth-check and endpoint TODOs.

**Readiness:** **External-input-dependent.** The repo already shows the debt, but honest closure still needs sanitized capture/session artifacts and selector confirmation that are not guaranteed by repo state alone.

**Files:**
- Modify: `backend/plugins/src/ecosystem/dlsite.rs`
- Modify: `backend/plugins/src/ecosystem/fanza.rs`
- Modify: `backend/plugins/tests/ecosystem_dlsite_fanza_test.rs`
- Modify: `backend/plugins/tests/fixtures/real/dlsite_library.json`
- Modify: `backend/plugins/tests/fixtures/real/fanza_library.json`
- Modify: `docs/har_extraction_guide.md`

- [ ] Confirm the actual auth-check URL, library endpoint, and required headers from sanitized captured HAR/session data.
- [ ] Replace the placeholder `navigate(login)`/`?output=json` assumptions with verified request targets and selector-based session validation.
- [ ] Remove stale `TODO(network-inspection)` comments once the code reflects the captured flow.
- [ ] Update `docs/har_extraction_guide.md` with the verified DLSite/FANZA capture steps, headers, selectors, and sanitization notes.
- [ ] Expand regression coverage for selector presence, cookie-header usage, parser structure, and non-200 responses.
- [ ] Verify repo-local parser/regression coverage with `cd backend && cargo test -p plugins ecosystem_dlsite_fanza_test -- --nocapture && cargo test -p plugins`, but do not call the task done until the capture-backed auth/library assumptions are confirmed from external artifacts.

### Task P10-A2: Finish Kindle Browser Sync Hardening

**Goal:** Make the Kindle connector honest and production-safe: either fully implement the verified Amazon browser/library path or codify a clear fallback contract instead of the current half-real placeholder flow.

**Readiness:** **External-input-dependent.** Repo-local tests can cover parser and fallback behavior, but the real browser path still depends on captured Amazon flow details and any required operator-provided session material.

**Files:**
- Modify: `backend/plugins/src/ecosystem/kindle.rs`
- Modify: `backend/plugins/src/browser_session.rs` (only if a missing browser primitive blocks the real flow)
- Create: `backend/plugins/tests/ecosystem_kindle_test.rs`
- Modify: `docs/har_extraction_guide.md`

- [ ] Verify the real Kindle library endpoint and required request headers/cookies from sanitized captured traffic or exported-library flow.
- [ ] Replace `TODO(network-inspection)` auth and library placeholders with either a verified browser sync flow or an explicit “CSV-only browser fallback” contract.
- [ ] Keep OTP behavior aligned with the existing backend OTP endpoint; do not introduce a second flow.
- [ ] Add Kindle-specific parser/auth regression tests in `backend/plugins/tests/ecosystem_kindle_test.rs`.
- [ ] Update `docs/har_extraction_guide.md` with the chosen Kindle sync path, fallback rules, and required captured artifacts.
- [ ] Verify repo-local parser/fallback coverage with `cd backend && cargo test -p plugins ecosystem_kindle_test -- --nocapture && cargo test -p plugins`, but do not claim real-flow closure without the external capture/export input.

### Task P10-A3: Add Real BookWalker Integration

**Goal:** Move BookWalker from deferred placeholder to a real connector with captured fixtures, verified selectors, and sync-ready library parsing.

**Readiness:** **External-input-dependent.** This task is blocked until sanitized BookWalker capture/session data exists outside git; repo-only edits are not enough to make the connector honest.

**Files:**
- Modify: `backend/plugins/src/ecosystem/bookwalker.rs`
- Create: `backend/plugins/tests/ecosystem_bookwalker_test.rs`
- Create: `backend/plugins/tests/fixtures/real/bookwalker_library.json`
- Modify: `docs/har_extraction_guide.md`

- [ ] Capture or obtain sanitized BookWalker HAR/session data and store a real-structure fixture.
- [ ] Replace placeholder login selectors and placeholder library URL assumptions with verified ones.
- [ ] Add parser/auth regression tests covering JSON shape, login form selectors, and non-success responses.
- [ ] Extend `docs/har_extraction_guide.md` with BookWalker-specific capture and sanitization steps so the connector can be refreshed later.
- [ ] Verify repo-local parser/auth regression coverage with `cd backend && cargo test -p plugins ecosystem_bookwalker_test -- --nocapture && cargo test -p plugins`, but keep the task blocked until the external capture/session input exists.

---

## Track B — Search UX Completion

### Task P10-B1: Make Search History Durable

**Goal:** Upgrade search history from in-memory-only behavior to app-restart durability without breaking the existing pure service contract.

**Readiness:** **Repo-local executable.** The gap is visible in committed code and can be implemented and verified from the repo alone.

**Files:**
- Modify: `frontend/lib/services/search_history_service.dart`
- Create: `frontend/lib/services/search_history_storage.dart`
- Modify: `frontend/lib/main.dart`
- Modify: `CONTEXT.md`
- Test: `frontend/test/services/search_history_service_test.dart`
- Create: `frontend/test/services/search_history_storage_test.dart`

- [ ] Introduce a storage seam so `SearchHistoryService` can load/save history instead of keeping everything process-local.
- [ ] Preserve the existing max-history and LIFO behavior.
- [ ] Keep serialization compatible with the existing `SearchHistory` JSON shape.
- [ ] Record the durable history contract and storage rules in `CONTEXT.md`.
- [ ] Verify with `cd frontend && flutter test test/services/search_history_service_test.dart test/services/search_history_storage_test.dart && flutter build apk --debug && flutter test`

### Task P10-B2: Add Search History Replay UI

**Goal:** Let users see recent searches, remove them, clear them, and replay them into the current list/search flow.

**Readiness:** **Repo-local executable.** This depends on `P10-B1`, not on outside captures or secrets.

**Files:**
- Create: `frontend/lib/widgets/search_history_panel.dart`
- Modify: `frontend/lib/screens/resource_list_screen.dart`
- Modify: `frontend/lib/widgets/search_filter_bar.dart`
- Modify: `CONTEXT.md`
- Create: `frontend/test/widgets/search_history_panel_test.dart`
- Create: `frontend/test/screens/resource_list_screen_search_history_test.dart`

- [ ] Render recent searches near the existing filter controls instead of leaving history as a hidden service.
- [ ] Replaying a saved search must restore query/tags/sort/filter logic together, not only the free-text query.
- [ ] Support per-item delete and full clear behavior.
- [ ] Document replay behavior, history limits, and clear/remove semantics in `CONTEXT.md`.
- [ ] Verify with `cd frontend && flutter test test/widgets/search_history_panel_test.dart test/screens/resource_list_screen_search_history_test.dart && flutter build apk --debug && flutter test`

### Task P10-B3: Surface Facets in the UI

**Goal:** Use the backend’s existing facet support so users can see and apply format-oriented refinements instead of only raw tag chips.

**Readiness:** **Repo-local executable.** Do this after the history durability/replay wave so the list-screen extension work does not sprawl in two directions at once.

**Files:**
- Create: `frontend/lib/widgets/facet_summary_bar.dart`
- Modify: `frontend/lib/screens/resource_list_screen.dart`
- Modify: `frontend/lib/repositories/ebook_repository.dart`
- Modify: `frontend/lib/repositories/image_repository.dart`
- Modify: `frontend/lib/repositories/video_repository.dart`
- Modify: `frontend/lib/repositories/game_repository.dart`
- Modify: `frontend/lib/repositories/web_reader_repository.dart`
- Modify: `CONTEXT.md`
- Create: `frontend/test/widgets/facet_summary_bar_test.dart`
- Create: `frontend/test/screens/resource_list_screen_facets_test.dart`

- [ ] Request and parse facet-aware list responses without breaking the plain-array response path.
- [ ] Render returned format counts as tappable facet chips or counters in the list UI.
- [ ] Keep facet state aligned with the existing `SearchFilterBloc` so refinements survive tab reloads.
- [ ] Document the facet response shape and UI refinement rules in `CONTEXT.md`.
- [ ] Verify with `cd frontend && flutter test test/widgets/facet_summary_bar_test.dart test/screens/resource_list_screen_facets_test.dart && flutter build apk --debug && flutter test`

### Task P10-B4: Add Tag Autocomplete / Typeahead

**Goal:** Replace plain free-text tag entry with suggestion-aware input in both search filters and resource tag editing.

**Readiness:** **Repo-local executable after earlier search UI work lands.** Do not start this in the first ready wave; it depends on the search-history durability/replay path first and should wait until that list-screen work has settled.

**Files:**
- Create: `frontend/lib/widgets/tag_autocomplete_field.dart`
- Modify: `frontend/lib/widgets/search_filter_bar.dart`
- Modify: `frontend/lib/widgets/tag_chip_list.dart`
- Modify: `CONTEXT.md`
- Create: `frontend/test/widgets/tag_autocomplete_field_test.dart`
- Test: `frontend/test/widgets/tag_chip_list_test.dart`

- [ ] Reuse existing tag models instead of creating a second tag shape.
- [ ] Suggest known tags while still allowing a new tag to be created explicitly.
- [ ] Keep current chip add/remove behavior working for both detail and search flows.
- [ ] Document suggestion, creation, and duplicate-handling rules in `CONTEXT.md`.
- [ ] Verify with `cd frontend && flutter test test/widgets/tag_autocomplete_field_test.dart test/widgets/tag_chip_list_test.dart && flutter build apk --debug && flutter test`

---

## Track C — Mobile Release Readiness

### Task P10-C1: Harden Android Release Identity and Signing

**Goal:** Replace the example Android identity and debug-signed release build with an env-driven, documented release path.

**Readiness:** **Secret/config-dependent.** The Gradle/docs/CI changes are repo-local, but honest release-path validation still depends on ignored local signing files and/or CI secrets.

**Files:**
- Modify: `frontend/android/app/build.gradle.kts`
- Modify: `docs/build-android.md`
- Modify: `.github/workflows/mobile-builds.yml`
- Test: `frontend/test/android_build_test.dart`

- [ ] Replace the example `applicationId` with a repo-owned configurable value.
- [ ] Load release-signing settings from ignored local files and/or CI secrets instead of using debug signing for release.
- [ ] Keep debug APK verification intact while adding a release-path guardrail for configured environments.
- [ ] Expand `docs/build-android.md` so the local and CI release paths use the same signing/config contract.
- [ ] Verify repo-local guards with `cd frontend && flutter build apk --debug && flutter test test/android_build_test.dart && flutter test`; only count release-path smoke as complete when ignored signing config or CI secrets are actually configured.

### Task P10-C2: Add iOS Device-Signing and Release-Export Runbook

**Goal:** Move iOS from “infrastructure exists” to a repeatable device/release process with concrete repo docs and CI guardrails.

**Readiness:** **Secret/config-dependent.** The runbook and config checks are repo-local, but device/archive/export validation still needs Apple signing assets outside git.

**Files:**
- Create: `docs/build-ios-device.md`
- Modify: `.github/workflows/mobile-builds.yml`
- Test: `frontend/test/ios_build_config_test.dart`

- [ ] Document the exact provisioning, signing, bundle-ID, and archive/export steps needed for this project.
- [ ] Add CI checks that validate the presence and shape of any repo-tracked iOS configuration files that the runbook depends on.
- [ ] Keep secrets and signing assets out of git; document only the expected file names and environment variables.
- [ ] Verify repo-local config/docs guards with `cd frontend && flutter test test/ios_build_config_test.dart && flutter build apk --debug && flutter test`; keep archive/export validation conditional on real Apple signing assets.

---

## Track D — Performance and Batch-Flow Polish

### Task P10-D1: Profile and Optimize Resource List Rendering

**Goal:** Replace vague “performance profiling” with a measured pass over the heaviest list/filter flows and implement only the fixes proven to matter.

**Readiness:** **Repo-local executable, but intentionally second-wave.** Keep it out of the first wave until the concrete search/batch follow-up work stops moving `resource_list_screen.dart` and `search_filter_bar.dart`, otherwise the profile data will be stale before the fixes land.

**Files:**
- Modify: `frontend/lib/screens/resource_list_screen.dart`
- Modify: `frontend/lib/widgets/search_filter_bar.dart`
- Modify: `frontend/lib/blocs/search_filter/search_filter_bloc.dart`
- Modify: `CONTEXT.md`

- [ ] Capture baseline timings for tab switch, filter apply, and large-list rebuilds in profile mode.
- [ ] Fix only the hot spots confirmed by the measurements (for example: unnecessary rebuilds, repeated fetches, or large synchronous transforms).
- [ ] Keep the optimization local; do not redesign the whole search stack unless the profile data demands it.
- [ ] Record the measured baseline, chosen fixes, and post-fix numbers in `CONTEXT.md`.
- [ ] Verify with `cd frontend && flutter test`

### Task P10-D2: Polish Batch Operations UX

**Goal:** Upgrade batch operations from raw comma-separated text fields to a safer, lower-friction bulk workflow.

**Readiness:** **Repo-local executable.** The current CSV/text-field debt is in committed UI code and can be tightened without waiting on outside inputs.

**Files:**
- Modify: `frontend/lib/screens/batch_operations_screen.dart`
- Modify: `frontend/lib/widgets/loading_widgets.dart`
- Modify: `CONTEXT.md`
- Test: `frontend/test/screens/batch_operations_screen_test.dart`
- Create: `frontend/test/screens/batch_operations_validation_test.dart`
- Test: `frontend/test/widgets/loading_widgets_test.dart`

- [ ] Replace the most error-prone raw text-field flows with clearer affordances where the current UI is obviously brittle.
- [ ] Surface validation and progress feedback inline instead of making batch actions feel fire-and-forget.
- [ ] Preserve the existing backend API shape; this is polish on top of working endpoints, not a new protocol.
- [ ] Document the final batch validation and progress rules in `CONTEXT.md`.
- [ ] Verify with `cd frontend && flutter test test/screens/batch_operations_screen_test.dart test/screens/batch_operations_validation_test.dart test/widgets/loading_widgets_test.dart && flutter build apk --debug && flutter test`

---

## Track E — Repo Task Tooling

### Task P10-E1: Add Cross-OS Repo Task Entrypoints

**Goal:** Stop making every contributor memorize raw command sequences. Add blunt repo-local `bin/app` and `bin/app.ps1` dispatchers so common backend/frontend commands stop living as scattered tribal knowledge.

**Readiness:** **Repo-local executable** with **host-limited validation**. The scripts and docs can be built from this repo, but Windows PowerShell smoke still needs a Windows host before anyone claims full cross-OS validation.

**Files:**
- Create: `bin/app`
- Create: `bin/app.ps1`
- Optional create: `bin/lib.sh`
- Optional create: `bin/lib.ps1`
- Modify: `README.md`
- Modify: `CONTEXT.md`
- Reference: `docs/superpowers/specs/2026-04-22-bin-dispatcher-design.md`

- [ ] Add `bin/app` as the POSIX dispatcher for macOS and Arch Linux, keeping it `sh`-compatible instead of bash-heavy.
- [ ] Add `bin/app.ps1` as the Windows-native PowerShell dispatcher with the same command contract.
- [ ] Support `start backend`, `start frontend <macos|windows|linux|ios|android>`, `build backend`, `build frontend <macos|windows|linux|ios|android>`, `test <backend|frontend|all>`, `clean`, and `gen-api`.
- [ ] Copy backend and frontend release artifacts into `dist/` without changing the underlying Cargo/Flutter commands the dispatchers wrap.
- [ ] Wire frontend verification so the existing repo rule stays intact: `flutter build apk --debug` before `flutter test`.
- [ ] Fail loudly on invalid verb/target pairs and unsupported host/target combinations; do not add fake cross-compilation behavior.
- [ ] Keep the shell and PowerShell entrypoints behaviorally aligned and blunt about prerequisites, and document the contract in `README.md` and `CONTEXT.md`.
- [ ] Verify POSIX smoke locally from this repo, and mark Windows PowerShell smoke as host-limited until it runs on a Windows machine or equivalent PowerShell-capable host.

---

## Execution Order Summary

```
Already complete at base SHA (do not execute in the next wave):
  P10-A0

Ready wave executed in this repo:
  P10-E1
  P10-B1 -> P10-B2
  P10-C1
  P10-D2

Next repo-local follow-up:
  P10-B3 -> P10-B4

Blocked / conditional tracks:
  P10-A1 -> P10-A2 -> P10-A3
    - Needs sanitized external capture/session input before closure is honest.

  P10-C2
    - Needs real Apple signing/export assets plus a concrete runbook before closure is honest.
  P10-D1
    - Needs honest profile/baseline data before any optimization claim is real.
  P10-E1
    - Windows PowerShell smoke stays host-limited until it runs on a Windows-capable host.
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
8. Every Phase 10 track lands repo-facing documentation and stronger regression coverage, not only narrow happy-path changes.
9. The repo has one clear testing reference (`docs/testing-matrix.md`) for the commands and prerequisites that Phase 10 relies on.
10. The repo has one clear cross-OS task entrypoint via `bin/app` and `bin/app.ps1`, including `dist/` copy behavior for backend and frontend build outputs, and Windows PowerShell smoke is tracked as host-limited until actually run on the right host.
