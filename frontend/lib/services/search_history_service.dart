import '../models/search_history.dart';
import 'search_history_storage.dart';

class SearchHistoryService {
  SearchHistoryService({
    SearchHistoryStorage? storage,
    int maxHistory = 50,
  })  : _storage = storage ?? const NoopSearchHistoryStorage(),
        _maxHistory = maxHistory,
        _history = [];

  final SearchHistoryStorage _storage;
  final int _maxHistory;
  final List<SearchHistory> _history;

  List<SearchHistory> get history => List.unmodifiable(_history);

  Future<void> load() async {
    final loadedHistory = await _storage.load();
    final trimmedHistory = _trimHistory(loadedHistory);

    _history
      ..clear()
      ..addAll(trimmedHistory);

    if (trimmedHistory.length != loadedHistory.length) {
      await _storage.save(history);
    }
  }

  Future<void> addSearch({
    required String query,
    required List<String> tags,
    required String sortBy,
    required String filterLogic,
  }) async {
    final now = DateTime.now();
    final search = SearchHistory(
      id: '${now.millisecondsSinceEpoch}',
      query: query,
      tags: tags,
      sortBy: sortBy,
      filterLogic: filterLogic,
      timestamp: now,
    );

    _history.insert(0, search);
    _enforceMaxHistory();
    await _storage.save(history);
  }

  SearchHistory? getById(String id) {
    try {
      return _history.firstWhere((h) => h.id == id);
    } catch (e) {
      return null;
    }
  }

  Future<void> removeById(String id) async {
    final initialLength = _history.length;
    _history.removeWhere((h) => h.id == id);
    if (_history.length != initialLength) {
      await _storage.save(history);
    }
  }

  Future<void> clearHistory() async {
    _history.clear();
    await _storage.save(history);
  }

  void _enforceMaxHistory() {
    if (_history.length > _maxHistory) {
      _history.removeRange(_maxHistory, _history.length);
    }
  }

  List<SearchHistory> _trimHistory(List<SearchHistory> entries) {
    if (entries.length <= _maxHistory) {
      return List<SearchHistory>.from(entries);
    }

    return List<SearchHistory>.from(entries.take(_maxHistory));
  }
}
