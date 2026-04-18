import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/dedup.dart';

void main() {
  group('DedupWarningStatus', () {
    test('fromString parses known statuses', () {
      expect(DedupWarningStatus.fromString('pending'), DedupWarningStatus.pending);
      expect(DedupWarningStatus.fromString('dismissed'), DedupWarningStatus.dismissed);
      expect(DedupWarningStatus.fromString('merged'), DedupWarningStatus.merged);
    });

    test('fromString defaults to pending for unknown', () {
      expect(DedupWarningStatus.fromString('unknown'), DedupWarningStatus.pending);
    });
  });

  group('DedupWarning', () {
    test('fromJson parses all fields', () {
      final json = {
        'id': 'w1',
        'resource_id_a': 'a1',
        'resource_id_b': 'b1',
        'similarity_score': 0.92,
        'status': 'pending',
      };
      final warning = DedupWarning.fromJson(json);
      expect(warning.id, 'w1');
      expect(warning.resourceIdA, 'a1');
      expect(warning.resourceIdB, 'b1');
      expect(warning.similarityScore, 0.92);
      expect(warning.status, DedupWarningStatus.pending);
    });

    test('fromJson handles integer similarity_score', () {
      final json = {
        'id': 'w1',
        'resource_id_a': 'a1',
        'resource_id_b': 'b1',
        'similarity_score': 1,
        'status': 'merged',
      };
      final warning = DedupWarning.fromJson(json);
      expect(warning.similarityScore, 1.0);
    });

    test('equality based on fields', () {
      const a = DedupWarning(
        id: 'w1', resourceIdA: 'a1', resourceIdB: 'b1',
        similarityScore: 0.92, status: DedupWarningStatus.pending,
      );
      const b = DedupWarning(
        id: 'w1', resourceIdA: 'a1', resourceIdB: 'b1',
        similarityScore: 0.92, status: DedupWarningStatus.pending,
      );
      expect(a, equals(b));
    });
  });

  group('MergeInput', () {
    test('toJson produces correct keys', () {
      const input = MergeInput(keepId: 'a1', discardId: 'b1');
      expect(input.toJson(), {'keep_id': 'a1', 'discard_id': 'b1'});
    });
  });
}
