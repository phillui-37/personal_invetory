import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import '../models/search_history.dart';

abstract class SearchHistoryStorage {
  Future<List<SearchHistory>> load();

  Future<void> save(List<SearchHistory> history);
}

class SharedPreferencesSearchHistoryStorage implements SearchHistoryStorage {
  SharedPreferencesSearchHistoryStorage(this._preferences);

  static const storageKey = 'search_history';

  final SharedPreferences _preferences;

  @override
  Future<List<SearchHistory>> load() async {
    final raw = _preferences.getString(storageKey);
    if (raw == null || raw.isEmpty) {
      return const [];
    }

    try {
      final decoded = jsonDecode(raw) as List<dynamic>;
      return decoded
          .map(
            (entry) => SearchHistory.fromJson(
              Map<String, dynamic>.from(entry as Map),
            ),
          )
          .toList();
    } on FormatException {
      await _preferences.remove(storageKey);
      return const [];
    } on TypeError {
      await _preferences.remove(storageKey);
      return const [];
    }
  }

  @override
  Future<void> save(List<SearchHistory> history) async {
    final payload = jsonEncode(
      history.map((entry) => entry.toJson()).toList(),
    );
    await _preferences.setString(storageKey, payload);
  }
}

class NoopSearchHistoryStorage implements SearchHistoryStorage {
  const NoopSearchHistoryStorage();

  @override
  Future<List<SearchHistory>> load() async => const [];

  @override
  Future<void> save(List<SearchHistory> history) async {}
}
