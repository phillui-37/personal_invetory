import 'dart:io';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('CI/CD Pipeline Configuration', () {
    late File workflowFile;
    late String workflowContent;

    setUpAll(() {
      workflowFile = File('../.github/workflows/mobile-builds.yml');
      if (workflowFile.existsSync()) {
        workflowContent = workflowFile.readAsStringSync();
      }
    });

    test('Mobile builds workflow file exists', () {
      expect(workflowFile.existsSync(), isTrue,
        reason: '.github/workflows/mobile-builds.yml must be created for CI/CD');
    });

    test('Workflow has correct trigger events', () {
      expect(workflowContent.contains('push:'), isTrue,
        reason: 'Workflow should trigger on push events');
      expect(workflowContent.contains('pull_request:'), isTrue,
        reason: 'Workflow should trigger on pull request events');
    });

    test('Workflow includes build-apk job for Android', () {
      expect(workflowContent.contains('build-apk:'), isTrue,
        reason: 'Workflow must have build-apk job');
      expect(workflowContent.contains('flutter build apk'), isTrue,
        reason: 'build-apk job must execute flutter build apk');
    });

    test('Workflow includes verify-ios-config job', () {
      expect(workflowContent.contains('verify-ios-config:'), isTrue,
        reason: 'Workflow must have verify-ios-config job');
      expect(workflowContent.contains('runs-on: macos-latest'), isTrue,
        reason: 'iOS configuration verification must run on macOS');
    });

    test('Workflow includes flutter tests job', () {
      expect(workflowContent.contains('run-flutter-tests:'), isTrue,
        reason: 'Workflow must have run-flutter-tests job');
      expect(workflowContent.contains('flutter test'), isTrue,
        reason: 'Tests must be executed in CI pipeline');
    });

    test('APK build has verification steps', () {
      expect(workflowContent.contains('Verify APK structure'), isTrue,
        reason: 'APK build must include structure validation');
      expect(workflowContent.contains('classes.dex'), isTrue,
        reason: 'Must verify APK contains Dalvik bytecode');
      expect(workflowContent.contains('libflutter.so'), isTrue,
        reason: 'Must verify APK contains native Flutter library');
    });

    test('Workflow uploads build artifacts', () {
      expect(workflowContent.contains('upload-artifact'), isTrue,
        reason: 'Workflow should upload APK artifacts for debugging');
      expect(workflowContent.contains('android-apk'), isTrue,
        reason: 'APK artifact name must be identifiable');
    });

    test('Workflow runs on appropriate platforms', () {
      expect(workflowContent.contains('ubuntu-latest'), isTrue,
        reason: 'Android builds should run on Ubuntu');
      expect(workflowContent.contains('macos-latest'), isTrue,
        reason: 'iOS verification and tests should run on macOS');
    });
  });
}
