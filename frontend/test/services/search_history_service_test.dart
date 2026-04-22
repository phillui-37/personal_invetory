import 'package:flutter_test/flutter_test.dart';

import '../../lib/models/search_history.dart';
import '../../lib/services/search_history_service.dart';
import '../../lib/services/search_history_storage.dart';

class _FakeSearchHistoryStorage implements SearchHistoryStorage {
  _FakeSearchHistoryStorage({List<SearchHistory>? initialHistory})
      : _initialHistory = List<SearchHistory>.from(initialHistory ?? const []);

  final List<SearchHistory> _initialHistory;
  final List<List<SearchHistory>> savedSnapshots = [];

  @override
  Future<List<SearchHistory>> load() async =>
      List<SearchHistory>.from(_initialHistory);

  @override
  Future<void> save(List<SearchHistory> history) async {
    savedSnapshots.add(List<SearchHistory>.from(history));
  }
}

SearchHistory _historyEntry({
  required String id,
  required String query,
  required DateTime timestamp,
  List<String> tags = const [],
  String sortBy = 'title',
  String filterLogic = 'and',
}) {
  return SearchHistory(
    id: id,
    query: query,
    tags: tags,
    sortBy: sortBy,
    filterLogic: filterLogic,
    timestamp: timestamp,
  );
}

void main() {
  group('SearchHistoryService Tests', () {
    late SearchHistoryService service;
    late _FakeSearchHistoryStorage storage;

    setUp(() {
      storage = _FakeSearchHistoryStorage();
      service = SearchHistoryService(storage: storage);
    });

    test('initializes with empty history', () {
      expect(service.history, isEmpty);
    });

    test('loads existing history from storage', () async {
      final existingHistory = [
        _historyEntry(
          id: 'recent',
          query: 'recent query',
          tags: ['tag-1'],
          timestamp: DateTime.utc(2026, 4, 20, 12),
        ),
        _historyEntry(
          id: 'older',
          query: 'older query',
          timestamp: DateTime.utc(2026, 4, 19, 12),
        ),
      ];
      storage = _FakeSearchHistoryStorage(initialHistory: existingHistory);
      service = SearchHistoryService(storage: storage);

      await service.load();

      expect(service.history, existingHistory);
    });

    test('truncates oversized loaded history and saves trimmed snapshot',
        () async {
      final existingHistory = [
        _historyEntry(
          id: 'recent',
          query: 'query4',
          timestamp: DateTime.utc(2026, 4, 20, 12),
        ),
        _historyEntry(
          id: 'mid-1',
          query: 'query3',
          timestamp: DateTime.utc(2026, 4, 20, 11),
        ),
        _historyEntry(
          id: 'mid-2',
          query: 'query2',
          timestamp: DateTime.utc(2026, 4, 20, 10),
        ),
        _historyEntry(
          id: 'old',
          query: 'query1',
          timestamp: DateTime.utc(2026, 4, 20, 9),
        ),
      ];
      storage = _FakeSearchHistoryStorage(initialHistory: existingHistory);
      service = SearchHistoryService(storage: storage, maxHistory: 3);

      await service.load();

      expect(
        service.history.map((entry) => entry.query).toList(),
        ['query4', 'query3', 'query2'],
      );
      expect(storage.savedSnapshots, hasLength(1));
      expect(storage.savedSnapshots.single, service.history);
    });

    test('adds search to history and saves it', () async {
      await service.addSearch(
        query: 'test',
        tags: ['fiction'],
        sortBy: 'title',
        filterLogic: 'and',
      );

      expect(service.history, isNotEmpty);
      expect(service.history.first.query, 'test');
      expect(service.history.first.tags, ['fiction']);
      expect(storage.savedSnapshots, hasLength(1));
      expect(storage.savedSnapshots.single, service.history);
    });

    test('adds multiple searches in reverse chronological order', () async {
      await service.addSearch(
        query: 'first',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );
      await service.addSearch(
        query: 'second',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );

      expect(service.history.length, 2);
      expect(service.history.first.query, 'second');
      expect(service.history.last.query, 'first');
    });

    test('limits history to max size and saves truncated history', () async {
      final service = SearchHistoryService(storage: storage, maxHistory: 3);

      for (int i = 0; i < 5; i++) {
        await service.addSearch(
          query: 'query$i',
          tags: [],
          sortBy: 'title',
          filterLogic: 'and',
        );
      }

      expect(service.history.length, 3);
      expect(service.history.first.query, 'query4');
      expect(service.history.last.query, 'query2');
      expect(storage.savedSnapshots.last, service.history);
    });

    test('retrieves search by id', () async {
      await service.addSearch(
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

    test('removes search by id and saves remaining history', () async {
      await service.addSearch(
        query: 'test',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );

      final search = service.history.first;
      await service.removeById(search.id);

      expect(service.history, isEmpty);
      expect(storage.savedSnapshots.last, isEmpty);
    });

    test('clears all history and saves empty history', () async {
      await service.addSearch(
        query: 'test1',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );
      await service.addSearch(
        query: 'test2',
        tags: [],
        sortBy: 'title',
        filterLogic: 'and',
      );

      await service.clearHistory();

      expect(service.history, isEmpty);
      expect(storage.savedSnapshots.last, isEmpty);
    });

    test('stores and retrieves complex search with multiple tags', () async {
      await service.addSearch(
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
