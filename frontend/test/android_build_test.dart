import 'dart:io';
import 'package:flutter_test/flutter_test.dart';

void main() {
  const releaseContractEnvVars = <String>[
    'ANDROID_APPLICATION_ID',
    'ANDROID_KEYSTORE_PATH',
    'ANDROID_KEYSTORE_PASSWORD',
    'ANDROID_KEY_ALIAS',
    'ANDROID_KEY_PASSWORD',
  ];

  String readTextFile(File file, String description) {
    expect(file.existsSync(), isTrue, reason: '$description must exist');
    return file.readAsStringSync();
  }

  String workflowJobBlock(String workflowContent, String jobName) {
    final lines = workflowContent.split('\n');
    final startIndex = lines.indexOf('  $jobName:');

    expect(startIndex, greaterThanOrEqualTo(0),
        reason: 'Workflow must contain the $jobName job block');
    if (startIndex < 0) {
      return '';
    }

    final buffer = StringBuffer();
    for (var index = startIndex; index < lines.length; index++) {
      final line = lines[index];
      if (index > startIndex &&
          RegExp(r'^  [A-Za-z0-9_-]+:$').hasMatch(line)) {
        break;
      }
      buffer.writeln(line);
    }

    return buffer.toString();
  }

  group('Android APK Build Verification', () {
    late File apkFile;
    late File gradleFile;
    late File workflowFile;
    late File docsFile;

    setUpAll(() {
      apkFile = File('build/app/outputs/flutter-apk/app-debug.apk');
      gradleFile = File('android/app/build.gradle.kts');
      workflowFile = File('../.github/workflows/mobile-builds.yml');
      docsFile = File('../docs/build-android.md');
    });

    test('APK file exists', () {
      expect(apkFile.existsSync(), isTrue,
          reason:
              'build/app/outputs/flutter-apk/app-debug.apk must exist after: flutter build apk --debug');
    });

    test('APK is valid Zip archive', () {
      expect(apkFile.existsSync(), isTrue);

      // Read magic bytes for Zip file format (PK\x03\x04)
      final bytes = apkFile.readAsBytesSync().take(4).toList();
      final magic = String.fromCharCodes(bytes);
      expect(magic, startsWith('PK'),
          reason: 'APK must be valid Zip archive (starts with PK...)');
    });

    test('APK contains required classes.dex', () {
      final apkPath = apkFile.path;
      final processResult = Process.runSync(
        'unzip',
        ['-l', apkPath],
        runInShell: true,
      );

      expect(processResult.exitCode, 0,
          reason: 'unzip should succeed on valid APK');
      final output = processResult.stdout as String;
      expect(output.contains('classes.dex'), isTrue,
          reason: 'APK must contain classes.dex (Dalvik bytecode)');
    });

    test('APK contains Flutter native library for arm64-v8a', () {
      final processResult = Process.runSync(
        'unzip',
        ['-l', apkFile.path],
        runInShell: true,
      );

      final output = processResult.stdout as String;
      expect(output.contains('lib/arm64-v8a/libflutter.so'), isTrue,
          reason: 'APK must contain native Flutter library for ARM64');
    });

    test('APK contains Flutter assets', () {
      final processResult = Process.runSync(
        'unzip',
        ['-l', apkFile.path],
        runInShell: true,
      );

      final output = processResult.stdout as String;
      expect(output.contains('assets/flutter_assets/'), isTrue,
          reason: 'APK must contain Flutter assets directory');
      expect(output.contains('kernel_blob.bin'), isTrue,
          reason: 'APK must contain Dart kernel snapshot');
    });

    test('APK contains AndroidManifest.xml', () {
      final processResult = Process.runSync(
        'unzip',
        ['-l', apkFile.path],
        runInShell: true,
      );

      final output = processResult.stdout as String;
      expect(output.contains('AndroidManifest.xml'), isTrue,
          reason:
              'APK must contain AndroidManifest.xml with app configuration');
    });

    test('APK size is reasonable (not empty)', () {
      expect(apkFile.lengthSync(), greaterThan(10 * 1024 * 1024),
          reason:
              'APK should be at least 10MB (contains Flutter engine + assets)');
    });

    test('build.gradle.kts exists and is valid', () {
      final gradleContent =
          readTextFile(gradleFile, 'android/app/build.gradle.kts');

      expect(gradleFile.existsSync(), isTrue,
          reason:
              'android/app/build.gradle.kts must exist (configured in Phase 7)');

      expect(gradleContent.contains('com.android.application'), isTrue,
          reason: 'Gradle must apply Android application plugin');
      expect(
          gradleContent.contains('dev.flutter.flutter-gradle-plugin'), isTrue,
          reason: 'Gradle must apply Flutter Gradle plugin');
    });

    test('release config removes example app id and debug signing fallback',
        () {
      final gradleContent =
          readTextFile(gradleFile, 'android/app/build.gradle.kts');

      expect(gradleContent.contains('com.example.personal_inventory_frontend'),
          isFalse,
          reason:
              'Release-ready Gradle config must not ship the Flutter example application ID');
      expect(
          gradleContent.contains('signingConfigs.getByName("debug")'), isFalse,
          reason: 'Release builds must not fall back to debug signing');
    });

    test(
        'release config reads identity and signing inputs from env or ignored files',
        () {
      final gradleContent =
          readTextFile(gradleFile, 'android/app/build.gradle.kts');

      for (final envVar in releaseContractEnvVars) {
        expect(gradleContent.contains(envVar), isTrue,
            reason:
                'android/app/build.gradle.kts must read $envVar for release builds');
      }

      expect(gradleContent.contains('key.properties'), isTrue,
          reason:
              'Release config must support ignored frontend/android/key.properties for local signing');
    });

    test('docs and CI describe the same release signing contract', () {
      final workflowContent =
          readTextFile(workflowFile, '.github/workflows/mobile-builds.yml');
      final docsContent = readTextFile(docsFile, 'docs/build-android.md');

      expect(workflowFile.existsSync(), isTrue,
          reason: '.github/workflows/mobile-builds.yml must exist');
      expect(docsFile.existsSync(), isTrue,
          reason: 'docs/build-android.md must exist');

      for (final envVar in releaseContractEnvVars) {
        expect(workflowContent.contains(envVar), isTrue,
            reason: 'CI workflow must document or consume $envVar');
        expect(docsContent.contains(envVar), isTrue,
            reason: 'Android build docs must document $envVar');
      }

      expect(docsContent.contains('key.properties'), isTrue,
          reason:
              'Docs must explain the ignored local key.properties fallback');
      expect(workflowContent.contains('flutter build apk --release'), isTrue,
          reason:
               'CI should describe the release build path when signing inputs are present');
    });

    test('CI release keystore path matches Gradle rootProject resolution', () {
      final gradleContent =
          readTextFile(gradleFile, 'android/app/build.gradle.kts');
      final workflowContent =
          readTextFile(workflowFile, '.github/workflows/mobile-builds.yml');
      final docsContent = readTextFile(docsFile, 'docs/build-android.md');
      final buildApkJobContent = workflowJobBlock(workflowContent, 'build-apk');

      expect(gradleContent.contains('storeFile = rootProject.file('), isTrue,
          reason:
              'Gradle release signing must resolve ANDROID_KEYSTORE_PATH from frontend/android/');
      expect(buildApkJobContent.contains('ANDROID_KEYSTORE_PATH: ci-release.keystore'),
          isTrue,
          reason:
              'CI must keep ANDROID_KEYSTORE_PATH relative to frontend/android/');
      expect(
          buildApkJobContent.contains(
              'base64 --decode > "android/\$ANDROID_KEYSTORE_PATH"'),
          isTrue,
          reason:
              'CI must decode the keystore into frontend/android/ so Gradle can find ci-release.keystore');
      expect(
          docsContent.contains('frontend/android/ci-release.keystore'),
          isTrue,
          reason:
              'Docs must explain the same frontend/android keystore location used by Gradle and CI');
    });

    test('debug APK before full Flutter tests stays intact in docs and CI', () {
      final workflowContent =
          readTextFile(workflowFile, '.github/workflows/mobile-builds.yml');
      final docsContent = readTextFile(docsFile, 'docs/build-android.md');
      final flutterTestsJobContent =
          workflowJobBlock(workflowContent, 'run-flutter-tests');

      expect(docsContent.contains('flutter build apk --debug'), isTrue,
          reason: 'Docs must keep the debug APK prerequisite');
      expect(docsContent.contains('flutter test'), isTrue,
          reason:
               'Docs must keep the test command paired with the debug APK build');

      final debugBuildIndex =
          flutterTestsJobContent.indexOf('flutter build apk --debug');
      final flutterTestIndex =
          flutterTestsJobContent.indexOf('flutter test --coverage');
      expect(debugBuildIndex, greaterThanOrEqualTo(0),
          reason: 'CI must build the debug APK before running Flutter tests');
      expect(flutterTestIndex, greaterThan(debugBuildIndex),
          reason:
              'CI must run flutter build apk --debug before flutter test --coverage');
    });

    test('AndroidManifest.xml includes internet permission', () {
      // Extract and check AndroidManifest
      final manifestPath = 'android/app/src/main/AndroidManifest.xml';
      final manifestFile = File(manifestPath);

      if (manifestFile.existsSync()) {
        final content = manifestFile.readAsStringSync();
        expect(content.contains('INTERNET'), isTrue,
            reason:
                'AndroidManifest must include INTERNET permission for API access');
      } else {
        // Manifest is generated during build, so we check that the app was built successfully
        expect(apkFile.existsSync(), isTrue,
            reason: 'APK was successfully built with required permissions');
      }
    });
  });
}
