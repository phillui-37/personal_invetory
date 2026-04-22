# Android Build Guide

## Prerequisites

- Flutter SDK ≥ 3.x (`flutter --version`)
- Android SDK (API 34) installed via Android Studio or `sdkmanager`
- Java 17 (`java -version`)
- `ANDROID_HOME` env var pointing to the SDK directory

## Repo verification rule

Do not run the Flutter suite cold.

`frontend/test/android_build_test.dart` checks for `build/app/outputs/flutter-apk/app-debug.apk`, so every clean checkout or cleaned `build/` dir needs a fresh debug APK before `flutter test`.

```bash
cd frontend
flutter pub get
flutter build apk --debug
flutter test
```

CI now keeps the same contract.
`.github/workflows/mobile-builds.yml` builds `app-debug.apk` before `flutter test test/android_build_test.dart` and before the full `flutter test --coverage` job.
Treat the command block above as the local repo gate for this APK-before-tests contract.

## Build commands

```bash
cd frontend
flutter pub get
flutter build apk --debug          # debug APK
flutter build apk --release        # release APK (requires explicit identity + signing inputs)
```

Output: `frontend/build/app/outputs/flutter-apk/app-debug.apk`

## SDK targets

| Setting | Value |
|---|---|
| `minSdkVersion` | 21 (Android 5.0 Lollipop) |
| `targetSdkVersion` | 34 (Android 14) |
| `compileSdkVersion` | managed by Flutter (`flutter.compileSdkVersion`) |

## Permissions

| Permission | Reason |
|---|---|
| `INTERNET` | API calls + WebView |
| `READ_EXTERNAL_STORAGE` (≤ API 32) | Open local ebook/media files |
| `READ_MEDIA_IMAGES` / `READ_MEDIA_VIDEO` (API 33+) | Scoped storage access |

`android:usesCleartextTraffic="true"` is set for local HTTP development servers.
Remove or scope it for production builds.

## Release identity and signing

Release builds are now fail-fast on purpose.
They do **not** fall back to the debug keystore, and they do **not** ship the Flutter example app ID.

`frontend/android/app/build.gradle.kts` reads the release contract from either:

- ignored local files: `frontend/android/key.properties` or `frontend/android/local.properties`
- CI environment variables in `.github/workflows/mobile-builds.yml`

Required keys:

| Purpose | Local property key | CI env var |
|---|---|---|
| Android application ID | `applicationId` | `ANDROID_APPLICATION_ID` |
| Keystore file path | `storeFile` | `ANDROID_KEYSTORE_PATH` |
| Keystore password | `storePassword` | `ANDROID_KEYSTORE_PASSWORD` |
| Key alias | `keyAlias` | `ANDROID_KEY_ALIAS` |
| Key password | `keyPassword` | `ANDROID_KEY_PASSWORD` |

### Local release build

1. Generate a keystore:
   ```bash
   keytool -genkey -v -keystore frontend/android/release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias personal_inventory
   ```
2. Create `frontend/android/key.properties` (gitignored):
   ```
   applicationId=dev.phillui.personal_inventory
   storeFile=release.jks
   storePassword=<password>
   keyAlias=personal_inventory
   keyPassword=<password>
   ```
3. Build:
   ```bash
   cd frontend
   flutter build apk --release
   ```

If any required value is missing, the Gradle release path aborts with a clear error instead of reusing debug signing.

### CI release path

The workflow keeps debug verification unconditional, then only runs `flutter build apk --release` when these GitHub inputs exist:

- repo variable: `ANDROID_APPLICATION_ID`
- repo secrets: `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`

The workflow decodes `ANDROID_KEYSTORE_BASE64` into `frontend/android/ci-release.keystore`, exports `ANDROID_KEYSTORE_PATH=ci-release.keystore`, and then runs the release build. That path split is intentional: Gradle resolves `ANDROID_KEYSTORE_PATH` with `rootProject.file(...)`, so the env var stays relative to `frontend/android/` while the workflow writes the decoded keystore into that same directory.

That means repo-local tests can prove the contract is wired, but they cannot prove a real signed release without actual signing material.
