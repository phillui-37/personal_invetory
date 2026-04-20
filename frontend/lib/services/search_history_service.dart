import '../models/search_history.dart';

class SearchHistoryService {
  SearchHistoryService({int maxHistory = 50})
      : _maxHistory = maxHistory,
        _history = [];

  final int _maxHistory;
  final List<SearchHistory> _history;

  List<SearchHistory> get history => List.unmodifiable(_history);

  void addSearch({
    required String query,
    required List<String> tags,
    required String sortBy,
    required String filterLogic,
  }) {
    final search = SearchHistory(
      id: '${DateTime.now().millisecondsSinceEpoch}',
      query: query,
      tags: tags,
      sortBy: sortBy,
      filterLogic: filterLogic,
      timestamp: DateTime.now(),
    );

    _history.insert(0, search);
    if (_history.length > _maxHistory) {
      _history.removeRange(_maxHistory, _history.length);
    }
  }

  SearchHistory? getById(String id) {
    try {
      return _history.firstWhere((h) => h.id == id);
    } catch (e) {
      return null;
    }
  }

  void removeById(String id) {
    _history.removeWhere((h) => h.id == id);
  }

  void clearHistory() {
    _history.clear();
  }
}

