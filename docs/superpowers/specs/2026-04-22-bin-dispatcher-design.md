# Bin Dispatcher Design Spec — Cross-OS Repo Task Entry Points

**Date**: 2026-04-22
**Author**: Copilot + Phil
**Status**: Approved design

---

## 1. Goal

Add a repo-level `bin/` tool entrypoint that makes the common backend/frontend developer tasks obvious and repeatable across:

- **macOS**
- **Arch Linux**
- **Windows PowerShell**

The first version covers:

- start backend
- start frontend (`macos`, `windows`, `linux`, `ios`, `android`)
- build backend into `dist/`
- build frontend (`macos`, `windows`, `linux`, `ios`, `android`) into `dist/`
- obvious helpers: `test`, `clean`, `gen-api`

The scripts must call the repo's existing real tools and outputs, not invent a second build system.

---

## 2. Scope

### In Scope

- Create a new repo-level `bin/` directory
- Add a POSIX entrypoint for macOS and Arch Linux
- Add a PowerShell entrypoint for Windows
- Keep one command contract across all OS entrypoints
- Start backend via Cargo
- Start frontend targets via Flutter
- Build backend/frontend release artifacts and copy them into `dist/`
- Add small helper commands for testing, cleaning, and OpenAPI client generation
- Document the command contract and platform limitations

### Out of Scope

- Docker orchestration wrappers
- Custom package/archive formats beyond the native build outputs
- Fake cross-compilation support for unsupported host/target pairs
- Replacing existing CI workflows
- Release signing automation beyond whatever the existing Flutter/Xcode/Gradle flow already requires

---

## 3. Command Contract

Two OS-native entrypoints expose the same subcommands:

- `bin/app` — POSIX shell entrypoint for macOS and Arch Linux
- `bin/app.ps1` — PowerShell entrypoint for Windows

### Command Surface

```text
bin/app start backend
bin/app start frontend <macos|windows|linux|ios|android>

bin/app build backend
bin/app build frontend <macos|windows|linux|ios|android>

bin/app test <backend|frontend|all>
bin/app clean
bin/app gen-api
```

PowerShell uses the same arguments:

```powershell
./bin/app.ps1 start backend
./bin/app.ps1 build frontend windows
```

### Behavior Rules

1. Commands are explicit. No hidden defaults like "frontend means macOS today".
2. The scripts call only real repo tools: `cargo`, `flutter`, and the existing `frontend/scripts/gen-api-client.sh`.
3. Unsupported host/target combinations fail loudly with a direct error message.
4. `build` means "build the native release artifact, then copy it into `dist/`".
5. The dispatcher is a thin task runner, not a new build abstraction layer.

---

## 4. Platform Layout

### 4.1 POSIX entrypoint

`bin/app` should stay plain `sh`-compatible so it works on default macOS shells and standard Arch Linux environments without bash-specific junk.

If shared POSIX helpers are needed, keep them small:

- `bin/lib.sh`

### 4.2 Windows entrypoint

`bin/app.ps1` is the Windows-native entrypoint. Do not require Git Bash just to use repo tasks on Windows.

If shared PowerShell helpers are needed, keep them small:

- `bin/lib.ps1`

### 4.3 Consistency rule

Both entrypoints must expose the same verbs, targets, validation rules, and help text as closely as the shell environments allow.

---

## 5. Command Implementation Mapping

### 5.1 Start commands

| Command | Underlying command |
|---|---|
| `start backend` | `cd backend && cargo run -p app` |
| `start frontend macos` | `cd frontend && flutter run -d macos` |
| `start frontend windows` | `cd frontend && flutter run -d windows` |
| `start frontend linux` | `cd frontend && flutter run -d linux` |
| `start frontend ios` | `cd frontend && flutter run -d ios` |
| `start frontend android` | `cd frontend && flutter run -d android` |

### 5.2 Build commands

All build commands use **release mode** by default.

| Command | Underlying command |
|---|---|
| `build backend` | `cd backend && cargo build -p app --release` |
| `build frontend macos` | `cd frontend && flutter build macos --release` |
| `build frontend windows` | `cd frontend && flutter build windows --release` |
| `build frontend linux` | `cd frontend && flutter build linux --release` |
| `build frontend ios` | `cd frontend && flutter build ios --release` |
| `build frontend android` | `cd frontend && flutter build apk --release` |

### 5.3 Helper commands

| Command | Underlying command |
|---|---|
| `test backend` | `cd backend && cargo test` |
| `test frontend` | `cd frontend && flutter build apk --debug && flutter test` |
| `test all` | backend test gate, then frontend debug-APK-plus-test gate |
| `clean` | remove repo build outputs and `dist/` |
| `gen-api` | call existing `frontend/scripts/gen-api-client.sh` |

The frontend test command must preserve the repo rule that `flutter build apk --debug` runs before `flutter test` because `frontend/test/android_build_test.dart` expects the APK to exist.

---

## 6. `dist/` Output Contract

`dist/` contains copied native artifacts only. It does **not** define a new packaging format.

### Layout

```text
dist/
  backend/
    app
    app.exe                # on Windows hosts
  frontend/
    macos/
    windows/
    linux/
    ios/
    android/
```

### Copy Rules

- `build backend` copies the built backend executable from `backend/target/release/` into `dist/backend/`
- `build frontend macos` copies the produced macOS app bundle into `dist/frontend/macos/`
- `build frontend windows` copies the produced Windows release bundle into `dist/frontend/windows/`
- `build frontend linux` copies the produced Linux release bundle into `dist/frontend/linux/`
- `build frontend android` copies the release APK into `dist/frontend/android/`
- `build frontend ios` copies the produced iOS build output into `dist/frontend/ios/`

### Failure Rules

1. If the build command exits non-zero, the dispatcher exits non-zero.
2. If the build succeeds but the expected artifact path is missing, the dispatcher exits non-zero with a direct message naming the missing path.
3. The dispatcher does not claim success before the artifact is actually copied into `dist/`.

---

## 7. Cleaning Rules

`clean` should remove:

- repo-local `dist/`
- backend build output (`backend/target/`)
- frontend build output (`frontend/build/`)

It must not touch unrelated caches outside the repository or run destructive global cleanup commands.

---

## 8. Error Handling and UX

The scripts should be blunt and simple.

- Print the exact command being run
- Fail on invalid verb/target combinations
- Print short usage/help when arguments are missing
- Refuse unsupported host/target combinations with a clear message
- Avoid silent fallbacks

Examples:

- asking for `start frontend windows` from POSIX on macOS should fail clearly instead of attempting nonsense
- asking for `build frontend ios` without iOS tooling should fail with the upstream Flutter/Xcode error, not hide it

---

## 9. Implementation Notes

1. Prefer small shell functions over giant case statements duplicated everywhere.
2. Keep path calculations repo-root aware so commands work no matter where the user invokes the script from.
3. Reuse existing repo docs and command conventions instead of inventing new ones.
4. Add help text that lists all supported commands and targets.
5. Update repo docs so contributors know `bin/app` and `bin/app.ps1` are the preferred task entrypoints.

---

## 10. Acceptance Criteria

This design is satisfied when:

1. The repo has a `bin/` directory with `bin/app` and `bin/app.ps1`
2. Both entrypoints expose the same command contract
3. Backend start/build commands work through the dispatcher
4. Frontend start/build commands exist for `macos`, `windows`, `linux`, `ios`, and `android`
5. Build commands copy native outputs into `dist/`
6. `test`, `clean`, and `gen-api` are wired
7. The scripts fail clearly on unsupported or invalid combinations
8. The relevant repo docs mention the new bin entrypoints
