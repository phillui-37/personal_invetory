import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/progress.dart';

void main() {
  group('ResourceProgress', () {
    test('fromJson parses backend payload and toJson round-trips', () {
      final json = <String, dynamic>{
        'resource_id': 'res-1',
        'progress': 0.75,
        'notes': 'chapter 12',
        'updated_at': '2026-04-19T12:34:56Z',
      };

      final progress = ResourceProgress.fromJson(json);

      expect(
        progress,
        equals(
          ResourceProgress(
            resourceId: 'res-1',
            progress: 0.75,
            notes: 'chapter 12',
            updatedAt: DateTime.parse('2026-04-19T12:34:56Z'),
          ),
        ),
      );
      expect(progress.toJson(), json);
    });
  });
}
