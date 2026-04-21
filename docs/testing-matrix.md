# Testing Matrix

Repo-wide test map. No guessing. Run the narrow suite while iterating, then run the full gate before calling work done.

## Global rules

1. Backend repo-wide gate: `cd backend && cargo test`
2. Frontend repo-wide gate: `cd frontend && flutter build apk --debug && flutter test`
3. The Flutter gate **always** needs the debug APK first because `frontend/test/android_build_test.dart` reads `build/app/outputs/flutter-apk/app-debug.apk`.
4. Connector and capture-doc work is not done when only docs change; keep `docs/har_extraction_guide.md` aligned with the plugin suite it describes.

## Matrix

| Area | What it protects | Suite owner | Prerequisites | Exact command |
|---|---|---|---|---|
| Backend workspace regression | All shipped Rust crates (`domain`, `use_cases`, `plugins`, `services`, `infrastructure`, `adapters`, `app`) | `backend/` workspace | Rust toolchain installed | `cd backend && cargo test` |
| Backend service-level shipped flows | Batch metadata update/copy behavior, chapter checks, device, progress, and tag services | `backend/services` | Rust toolchain installed | `cd backend && cargo test -p services --test services_tdd` |
| Flutter full regression | Whole Flutter app plus build assertions in `test/android_build_test.dart` | `frontend/` package | Flutter SDK, Android SDK, Java 17, dependencies resolved | `cd frontend && flutter build apk --debug && flutter test` |
| Resource search/filter regressions | `ResourceListScreen` reload wiring and `SearchFilterBar` behavior for shipped tag/sort/filter flows | `frontend/lib/screens/resource_list_screen.dart` + `frontend/lib/widgets/search_filter_bar.dart` | Same Flutter prerequisites as full suite | `cd frontend && flutter test test/widgets/search_filter_bar_test.dart test/screens/resource_list_screen_test.dart` |
| Batch workflow regressions | Import/update/copy request parsing on the shipped batch screen | `frontend/lib/screens/batch_operations_screen.dart` | Same Flutter prerequisites as full suite | `cd frontend && flutter test test/screens/batch_operations_screen_test.dart` |
| Android build contract | Debug APK existence/shape plus doc/CI parity for Android packaging | `frontend/android/` + `.github/workflows/mobile-builds.yml` + `docs/build-android.md` | Flutter SDK, Android SDK, Java 17 | `cd frontend && flutter build apk --debug && flutter test test/android_build_test.dart` |
| Ecosystem capture audit | Current connector placeholders stay visible while the local HAR capture runbook stays aligned; at current HEAD this audit is expected to print the outstanding `TODO(network-inspection)` markers and the selector/cookie/sanitize references they depend on | `backend/plugins` + `docs/har_extraction_guide.md` | Local sanitized HAR/session data only; never commit raw captures | `cd backend && cargo test -p plugins && cd .. && rg -n "TODO\\(network-inspection\\)" backend/plugins/src/ecosystem && rg -n "selector|cookie|sanitize" backend/plugins/src/ecosystem docs/har_extraction_guide.md` |

## Release gate for this repo

Run this exact sequence before closing repo-wide doc/test work:

```bash
cd backend && cargo test
cd ../frontend && flutter build apk --debug && flutter test
```
