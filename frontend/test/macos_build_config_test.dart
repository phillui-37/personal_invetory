import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  group('macOS Build Infrastructure Configuration', () {
    for (final path in const [
      'macos/Runner/DebugProfile.entitlements',
      'macos/Runner/Release.entitlements',
    ]) {
      test('$path allows reading user-selected files', () {
        final entitlements = File('${Directory.current.path}/$path');
        expect(entitlements.existsSync(), isTrue, reason: '$path must exist');

        final content = entitlements.readAsStringSync();
        expect(
          content.contains('com.apple.security.app-sandbox'),
          isTrue,
          reason: '$path must stay sandboxed',
        );
        expect(
          content.contains('com.apple.security.files.user-selected.read-only'),
          isTrue,
          reason:
              '$path must allow user-selected file and directory reads so '
              'BulkImportScreen can import a picked macOS folder',
        );
      });
    }
  });
}
