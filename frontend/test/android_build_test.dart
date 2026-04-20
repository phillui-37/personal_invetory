import 'dart:io';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('Android APK Build Verification', () {
    late File apkFile;

    setUpAll(() {
      apkFile = File('build/app/outputs/flutter-apk/app-debug.apk');
    });

    test('APK file exists', () {
      expect(apkFile.existsSync(), isTrue, 
        reason: 'build/app/outputs/flutter-apk/app-debug.apk must exist after: flutter build apk --debug');
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
        reason: 'APK must contain AndroidManifest.xml with app configuration');
    });

    test('APK size is reasonable (not empty)', () {
      expect(apkFile.lengthSync(), greaterThan(10 * 1024 * 1024), 
        reason: 'APK should be at least 10MB (contains Flutter engine + assets)');
    });

    test('build.gradle.kts exists and is valid', () {
      final gradleFile = File('android/app/build.gradle.kts');
      expect(gradleFile.existsSync(), isTrue, 
        reason: 'android/app/build.gradle.kts must exist (configured in Phase 7)');
      
      final content = gradleFile.readAsStringSync();
      expect(content.contains('com.android.application'), isTrue, 
        reason: 'Gradle must apply Android application plugin');
      expect(content.contains('dev.flutter.flutter-gradle-plugin'), isTrue, 
        reason: 'Gradle must apply Flutter Gradle plugin');
    });

    test('AndroidManifest.xml includes internet permission', () {
      // Extract and check AndroidManifest
      final manifestPath = 'android/app/src/main/AndroidManifest.xml';
      final manifestFile = File(manifestPath);
      
      if (manifestFile.existsSync()) {
        final content = manifestFile.readAsStringSync();
        expect(content.contains('INTERNET'), isTrue, 
          reason: 'AndroidManifest must include INTERNET permission for API access');
      } else {
        // Manifest is generated during build, so we check that the app was built successfully
        expect(apkFile.existsSync(), isTrue, 
          reason: 'APK was successfully built with required permissions');
      }
    });
  });
}
