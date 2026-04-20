import 'package:flutter_test/flutter_test.dart';

import '../../lib/models/search_history.dart';
import '../../lib/services/search_history_service.dart';

void main() {
  group('SearchHistoryService Tests', () {
    late SearchHistoryService service;

    setUp(() {
      service = SearchHistoryService();
    });

    test('initializes with empty history', () {
      expect(service.history, isEmpty);
    });

    test('adds search to history', () {
      service.addSearch(
        query: 'test',
        tags: ['fiction'],
        sortBy: 'title',
        filterLogic: 'and',
      );

      expect(service.history, isNotEmpty);
      expect(service.history.first.query, 'test');
      expect(service.history.first.tags, ['fiction']);
    });

    test('adds multiple searches in reverse chronological order', () {
      service.addSearch(
        query: 'first',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );
      service.addSearch(
        query: 'second',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );

      expect(service.history.length, 2);
      expect(service.history.first.query, 'second');
      expect(service.history.last.query, 'first');
    });

    test('limits history to max size', () {
      final service = SearchHistoryService(maxHistory: 3);

      for (int i = 0; i < 5; i++) {
        service.addSearch(
          query: 'query$i',
          tags: [],
          sortBy: 'title',
          filterLogic: 'and',
        );
      }

      expect(service.history.length, 3);
      expect(service.history.first.query, 'query4');
      expect(service.history.last.query, 'query2');
    });

    test('retrieves search by id', () {
      service.addSearch(
        query: 'test',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );

      final search = service.history.first;
      final retrieved = service.getById(search.id);

      expect(retrieved, isNotNull);
      expect(retrieved!.query, 'test');
    });

    test('returns null for non-existent id', () {
      final retrieved = service.getById('non-existent-id');
      expect(retrieved, isNull);
    });

    test('removes search by id', () {
      service.addSearch(
        query: 'test',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );

      final search = service.history.first;
      service.removeById(search.id);

      expect(service.history, isEmpty);
    });

    test('clears all history', () {
      service.addSearch(
        query: 'test1',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );
      service.addSearch(
        query: 'test2',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );

      service.clearHistory();

      expect(service.history, isEmpty);
    });

    test('stores and retrieves complex search with multiple tags', () {
      service.addSearch(
        query: 'complex',
        tags: ['fiction', 'mystery', 'adventure'],
        sortBy: 'date_added',
        filterLogic: 'or',
      );

      final search = service.history.first;

      expect(search.query, 'complex');
      expect(search.tags, ['fiction', 'mystery', 'adventure']);
      expect(search.sortBy, 'date_added');
      expect(search.filterLogic, 'or');
    });
  });

  group('SearchHistory Serialization Tests', () {
    test('serializes to JSON', () {
      final search = SearchHistory(
        id: 'test-id',
        query: 'test',
        tags: ['fiction'],
        sortBy: 'title',
        filterLogic: 'and',
        timestamp: DateTime(2026, 4, 20, 12, 0, 0),
      );

      final json = search.toJson();

      expect(json['id'], 'test-id');
      expect(json['query'], 'test');
      expect(json['tags'], ['fiction']);
      expect(json['sortBy'], 'title');
    });

    test('deserializes from JSON', () {
      final json = {
        'id': 'test-id',
        'query': 'test',
        'tags': ['fiction'],
        'sortBy': 'title',
        'filterLogic': 'and',
        'timestamp': '2026-04-20T12:00:00.000Z',
      };

      final search = SearchHistory.fromJson(json);

      expect(search.id, 'test-id');
      expect(search.query, 'test');
      expect(search.tags, ['fiction']);
      expect(search.sortBy, 'title');
    });
  });
}
