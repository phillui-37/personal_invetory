import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/tag.dart';

void main() {
  group('Tag', () {
    test('fromJson parses backend payload and toJson round-trips', () {
      final json = <String, dynamic>{
        'id': 'tag-1',
        'name': 'sci-fi',
        'created_at': '2026-04-19T12:34:56Z',
      };

      final tag = Tag.fromJson(json);

      expect(
        tag,
        equals(
          Tag(
            id: 'tag-1',
            name: 'sci-fi',
            createdAt: DateTime.parse('2026-04-19T12:34:56Z'),
          ),
        ),
      );
      expect(tag.toJson(), json);
    });
  });
}
