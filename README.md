# Personal Inventory

Personal inventory system for tracking ebooks, web readers, images, videos, and games across devices, platforms, and storage locations.

## Current status

- **Phases 1-9**: implemented and merged
- **Phase 10 backlog**: recorded in `tasks/phase10.md`
- **Current focus**: repo-wide docs/testing complement, connector hardening, search UX follow-up, mobile release readiness, and polish work

## Project shape

| Path | Purpose |
|---|---|
| `backend/` | Rust workspace: domain, use cases, services, adapters, infrastructure, app |
| `frontend/` | Flutter app: models, repositories, BLoCs, screens, widgets |
| `docs/` | Operator and contributor guides |
| `tasks/` | Phase-by-phase implementation backlogs |
| `CONTEXT.md` | Living project summary, decisions, and phase state |

## Read this first

1. `CONTEXT.md` — current architecture, decisions, and phase history
2. `docs/testing-matrix.md` — exact verification commands and prerequisites
3. `tasks/phase10.md` — current remaining work

## Verification commands

```bash
cd backend && cargo test
cd ../frontend && flutter build apk --debug && flutter test
```

The Flutter suite is **not** safe to run cold. `frontend/test/android_build_test.dart` expects `build/app/outputs/flutter-apk/app-debug.apk`, so build the debug APK first.

## Key guides

| File | What it covers |
|---|---|
| `docs/testing-matrix.md` | Repo-wide test suites, prerequisites, and ownership |
| `docs/build-android.md` | Android build and local verification flow |
| `docs/har_extraction_guide.md` | Safe local HAR inspection and connector-capture rules |
| `docs/api_credentials.md` | How to obtain platform credentials and session data |
| `docs/CREDENTIALS_SECURITY_POLICY.md` | What credential material may and may not be stored or committed |

## Architecture snapshot

- **Backend**: Rust + axum + sqlx, hexagonal/clean architecture
- **Frontend**: Flutter + BLoC
- **API contract**: OpenAPI via `utoipa`
- **Database**: SQLite is the working runtime adapter today; PostgreSQL remains planned/hardened work
- **Auth**: single API key

## Guardrails

- Do not commit live credentials, raw HAR files, or session cookies.
- Keep docs aligned with the code and tests they describe.
- Treat `tasks/phase9.md` as historical backlog; use `tasks/phase10.md` for current remaining work.
