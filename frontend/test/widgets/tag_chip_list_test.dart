import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/tag.dart';
import 'package:personal_inventory_frontend/widgets/tag_chip_list.dart';

final _createdAt = DateTime.utc(2025, 1, 1);

void main() {
  group('TagChipList', () {
    testWidgets('renders each tag as a chip', (tester) async {
      final tags = [
        Tag(id: 't1', name: 'sci-fi', createdAt: _createdAt),
        Tag(id: 't2', name: 'fantasy', createdAt: _createdAt),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TagChipList(
              tags: tags,
              onRemove: (_) {},
              onAdd: (_) {},
            ),
          ),
        ),
      );

      expect(find.text('sci-fi'), findsOneWidget);
      expect(find.text('fantasy'), findsOneWidget);
    });

    testWidgets('calls onRemove with tagId when delete icon tapped', (tester) async {
      String? removedId;
      final tags = [Tag(id: 't1', name: 'sci-fi', createdAt: _createdAt)];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TagChipList(
              tags: tags,
              onRemove: (id) => removedId = id,
              onAdd: (_) {},
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('tag-chip-delete-t1')));
      await tester.pump();

      expect(removedId, 't1');
    });

    testWidgets('shows add-tag button and calls onAdd with entered name', (tester) async {
      String? addedName;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TagChipList(
              tags: const [],
              onRemove: (_) {},
              onAdd: (name) => addedName = name,
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('tag-add-button')));
      await tester.pumpAndSettle();

      await tester.enterText(find.byKey(const Key('tag-add-input')), 'horror');
      await tester.tap(find.byKey(const Key('tag-add-confirm')));
      await tester.pump();

      expect(addedName, 'horror');
    });

    testWidgets('shows empty state when no tags', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TagChipList(
              tags: const [],
              onRemove: (_) {},
              onAdd: (_) {},
            ),
          ),
        ),
      );

      expect(find.text('No tags'), findsOneWidget);
    });
  });
}
