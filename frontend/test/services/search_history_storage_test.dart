import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../lib/models/search_history.dart';
import '../../lib/services/search_history_storage.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('SharedPreferencesSearchHistoryStorage', () {
    late SharedPreferences preferences;
    late SharedPreferencesSearchHistoryStorage storage;

    setUp(() async {
      SharedPreferences.setMockInitialValues({});
      preferences = await SharedPreferences.getInstance();
      storage = SharedPreferencesSearchHistoryStorage(preferences);
    });

    test('loads existing history from storage', () async {
      final payload = jsonEncode([
        SearchHistory(
          id: 'saved-1',
          query: 'saved query',
          tags: ['fiction'],
          sortBy: 'title',
          filterLogic: 'and',
          timestamp: DateTime.utc(2026, 4, 20, 12),
        ).toJson(),
      ]);
      await preferences.setString(
        SharedPreferencesSearchHistoryStorage.storageKey,
        payload,
      );

      final loaded = await storage.load();

      expect(loaded, hasLength(1));
      expect(loaded.single.query, 'saved query');
      expect(loaded.single.tags, ['fiction']);
    });

    test('returns empty history when storage is empty', () async {
      expect(await storage.load(), isEmpty);
    });

    test('fails open to empty history and clears malformed payload', () async {
      await preferences.setString(
        SharedPreferencesSearchHistoryStorage.storageKey,
        '{not valid json',
      );

      final loaded = await storage.load();

      expect(loaded, isEmpty);
      expect(
        preferences
            .containsKey(SharedPreferencesSearchHistoryStorage.storageKey),
        isFalse,
      );
    });

    test('saves history with compatible SearchHistory JSON shape', () async {
      final history = [
        SearchHistory(
          id: 'saved-1',
          query: 'saved query',
          tags: ['fiction', 'mystery'],
          sortBy: 'date_added',
          filterLogic: 'or',
          timestamp: DateTime.utc(2026, 4, 20, 12),
        ),
      ];

      await storage.save(history);

      final raw = preferences.getString(
        SharedPreferencesSearchHistoryStorage.storageKey,
      );
      expect(raw, isNotNull);

      final decoded = jsonDecode(raw!) as List<dynamic>;
      expect(decoded.single, {
        'id': 'saved-1',
        'query': 'saved query',
        'tags': ['fiction', 'mystery'],
        'sortBy': 'date_added',
        'filterLogic': 'or',
        'timestamp': '2026-04-20T12:00:00.000Z',
      });

      final roundTrip = SearchHistory.fromJson(
        Map<String, dynamic>.from(decoded.single as Map),
      );
      expect(roundTrip, history.single);
    });
  });
}
