# Personal Inventory

Personal inventory system for tracking ebooks, web readers, images, videos, and games across devices, platforms, and storage locations.

## Current status

- **Phases 1-9**: implemented and merged
- **Phase 10 backlog**: recorded in `tasks/phase10.md`
- **Current focus**: repo task tooling, repo-wide docs/testing complement, connector hardening, search UX follow-up, mobile release readiness, and polish work

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

## Repo task entrypoints

Use the repo-local dispatcher instead of memorizing raw commands:

```bash
bin/app start backend
bin/app test all
bin/app build frontend android
```

Windows uses the PowerShell entrypoint:

```powershell
.\bin\app.ps1 start backend
.\bin\app.ps1 test all
.\bin\app.ps1 build frontend android
```

Supported contract:

```text
start backend
start frontend <macos|windows|linux|ios|android>
build backend
build frontend <macos|windows|linux|ios|android>
test <backend|frontend|all>
clean
gen-api
```

Today this repo only has Flutter platform folders for `android`, `ios`, and `macos`. `windows` and `linux` stay in the command contract, but the dispatcher fails clearly until those repo targets are added.

## Local dev with SQLite

For local development, do not use the Postgres value from `.env.example` as-is. Create a local `.env` in the repo root and point the backend at SQLite instead:

```dotenv
DATABASE_URL=sqlite://./inventory.db
API_KEY=dev-local-key
DEVICE_ID=550e8400-e29b-41d4-a716-446655440000
HOST=0.0.0.0
PORT=8080
PLUGINS_CONFIG=plugins.toml
```

Notes:

1. `DATABASE_URL` and `DEVICE_ID` are required.
2. `API_KEY` should be set up front for a smooth dev loop. If you leave it blank or omit it, the backend will generate one, append it to `.env`, and exit once; then you rerun the backend.
3. `bin/app start backend` is fine for the backend, but frontend local API dev still needs raw `flutter run` so you can pass compile-time `--dart-define` values.

Start the backend:

```bash
bin/app start backend
```

Run the Flutter app against that local backend:

```bash
cd frontend
flutter run -d macos \
  --dart-define=BASE_URL=http://127.0.0.1:8080 \
  --dart-define=API_KEY=dev-local-key
```

For iOS simulator, keep the same `BASE_URL`. For the Android emulator, use `http://10.0.2.2:8080` instead of `http://127.0.0.1:8080`.

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
