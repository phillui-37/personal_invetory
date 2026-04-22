import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/search_history.dart';
import 'package:personal_inventory_frontend/widgets/search_history_panel.dart';

SearchHistory _entry({
  required String id,
  required String query,
  required List<String> tags,
  required String sortBy,
  required String filterLogic,
}) {
  return SearchHistory(
    id: id,
    query: query,
    tags: tags,
    sortBy: sortBy,
    filterLogic: filterLogic,
    timestamp: DateTime.utc(2026, 4, 22, 12, 0),
  );
}

void main() {
  group('SearchHistoryPanel', () {
    testWidgets('renders recent history entries', (tester) async {
      final history = [
        _entry(
          id: 'recent-1',
          query: 'alpha',
          tags: ['favorite'],
          sortBy: 'title',
          filterLogic: 'and',
        ),
        _entry(
          id: 'recent-2',
          query: 'beta',
          tags: ['archive'],
          sortBy: 'date_added',
          filterLogic: 'or',
        ),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchHistoryPanel(
              history: history,
              onReplay: (_) {},
              onRemove: (_) {},
              onClear: () {},
            ),
          ),
        ),
      );

      expect(find.text('Recent searches'), findsOneWidget);
      expect(find.byKey(const Key('search-history-item-recent-1')),
          findsOneWidget);
      expect(find.byKey(const Key('search-history-item-recent-2')),
          findsOneWidget);
      expect(find.text('alpha'), findsOneWidget);
      expect(find.text('beta'), findsOneWidget);
    });

    testWidgets('replays and removes a selected history entry', (tester) async {
      final entry = _entry(
        id: 'saved-1',
        query: 'alpha',
        tags: ['favorite'],
        sortBy: 'title',
        filterLogic: 'or',
      );
      SearchHistory? replayed;
      String? removedId;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchHistoryPanel(
              history: [entry],
              onReplay: (value) => replayed = value,
              onRemove: (value) => removedId = value,
              onClear: () {},
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('search-history-item-saved-1')));
      await tester.pump();
      expect(replayed, entry);

      await tester.tap(find.byKey(const Key('search-history-remove-saved-1')));
      await tester.pump();
      expect(removedId, 'saved-1');
    });

    testWidgets('clears all history entries', (tester) async {
      var cleared = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SearchHistoryPanel(
              history: [
                _entry(
                  id: 'saved-1',
                  query: 'alpha',
                  tags: ['favorite'],
                  sortBy: 'title',
                  filterLogic: 'and',
                ),
              ],
              onReplay: (_) {},
              onRemove: (_) {},
              onClear: () => cleared = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('search-history-clear-all')));
      await tester.pump();

      expect(cleared, isTrue);
    });
  });
}
