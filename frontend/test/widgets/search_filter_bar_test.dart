import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../lib/widgets/search_filter_bar.dart';
import '../../lib/models/tag.dart';

void main() {
  group('SearchFilterBar Widget Tests', () {
    final List<Tag> testTags = [
      Tag(id: '1', name: 'fiction', createdAt: DateTime.now()),
      Tag(id: '2', name: 'mystery', createdAt: DateTime.now()),
      Tag(id: '3', name: 'adventure', createdAt: DateTime.now()),
    ];

    testWidgets('renders tag filter chips', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchFilterBar(
              tags: testTags,
              selectedTags: [],
              onTagsChanged: (_) {},
              onSortChanged: (_) {},
              onLogicChanged: (_) {},
            ),
          ),
        ),
      );

      for (final tag in testTags) {
        expect(find.byKey(Key('filter-tag-${tag.name}')), findsOneWidget);
      }
    });

    testWidgets('renders tag chips list key', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchFilterBar(
              tags: testTags,
              selectedTags: [],
              onTagsChanged: (_) {},
              onSortChanged: (_) {},
              onLogicChanged: (_) {},
            ),
          ),
        ),
      );

      expect(find.byKey(const Key('tag-chips-list')), findsOneWidget);
    });

    testWidgets('shows clear filters button when tags selected',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchFilterBar(
              tags: testTags,
              selectedTags: ['fiction'],
              onTagsChanged: (_) {},
              onSortChanged: (_) {},
              onLogicChanged: (_) {},
            ),
          ),
        ),
      );

      expect(find.byKey(const Key('clear-filters-chip')), findsOneWidget);
    });

    testWidgets('shows sort and logic dropdowns with selected tags',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchFilterBar(
              tags: testTags,
              selectedTags: ['fiction'],
              onTagsChanged: (_) {},
              onSortChanged: (_) {},
              onLogicChanged: (_) {},
            ),
          ),
        ),
      );

      expect(find.byKey(const Key('sort-dropdown')), findsOneWidget);
      expect(find.byKey(const Key('logic-dropdown')), findsOneWidget);
    });

    testWidgets('hides sort and logic dropdowns without selected tags',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchFilterBar(
              tags: testTags,
              selectedTags: [],
              onTagsChanged: (_) {},
              onSortChanged: (_) {},
              onLogicChanged: (_) {},
            ),
          ),
        ),
      );

      expect(find.byKey(const Key('sort-dropdown')), findsNothing);
      expect(find.byKey(const Key('logic-dropdown')), findsNothing);
    });

    testWidgets('clear chip resets tags sort and logic to repo defaults',
        (WidgetTester tester) async {
      List<String>? changedTags;
      String? changedSort;
      String? changedLogic;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchFilterBar(
              tags: testTags,
              selectedTags: ['fiction'],
              sortOption: 'title',
              filterLogic: 'or',
              onTagsChanged: (tags) => changedTags = tags,
              onSortChanged: (sort) => changedSort = sort,
              onLogicChanged: (logic) => changedLogic = logic,
            ),
          ),
        ),
      );

      final clearChip = tester.widget<InputChip>(
        find.byKey(const Key('clear-filters-chip')),
      );
      clearChip.onDeleted!.call();
      await tester.pump();

      expect(changedTags, isEmpty);
      expect(changedSort, 'date_added');
      expect(changedLogic, 'and');
    });
  });
}
