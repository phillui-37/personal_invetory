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

Current CI does **not** run `flutter test` in the same job after the debug APK build.
`.github/workflows/mobile-builds.yml` validates the APK in `build-apk`, then runs `flutter test --coverage` separately in `run-flutter-tests`.
Treat the command block above as the local repo gate for this APK-before-tests contract.

## Build commands

```bash
cd frontend
flutter pub get
flutter build apk --debug          # debug APK
flutter build apk --release        # release APK (requires signing config)
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

## Release signing

1. Generate a keystore: `keytool -genkey -v -keystore release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias key`
2. Add `key.properties` to `frontend/android/` (gitignored):
   ```
   storePassword=<password>
   keyPassword=<password>
   keyAlias=key
   storeFile=../../release.jks
   ```
3. Reference it in `android/app/build.gradle.kts` under `signingConfigs`.
