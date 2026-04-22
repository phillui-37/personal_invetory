import 'package:flutter/material.dart';

import '../models/search_history.dart';

class SearchHistoryPanel extends StatelessWidget {
  const SearchHistoryPanel({
    required this.history,
    required this.onReplay,
    required this.onRemove,
    required this.onClear,
    super.key,
  });

  final List<SearchHistory> history;
  final ValueChanged<SearchHistory> onReplay;
  final ValueChanged<String> onRemove;
  final VoidCallback onClear;

  @override
  Widget build(BuildContext context) {
    if (history.isEmpty) {
      return const SizedBox.shrink();
    }

    return Card(
      margin: const EdgeInsets.fromLTRB(12, 0, 12, 12),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Expanded(
                  child: Text(
                    'Recent searches',
                    style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
                  ),
                ),
                TextButton(
                  key: const Key('search-history-clear-all'),
                  onPressed: onClear,
                  child: const Text('Clear all'),
                ),
              ],
            ),
            const SizedBox(height: 8),
            ...history.map(_buildHistoryItem),
          ],
        ),
      ),
    );
  }

  Widget _buildHistoryItem(SearchHistory entry) {
    return ListTile(
      key: Key('search-history-item-${entry.id}'),
      contentPadding: EdgeInsets.zero,
      title: Text(entry.query.isEmpty ? 'Saved tag filter' : entry.query),
      subtitle: Text(_buildSubtitle(entry)),
      trailing: IconButton(
        key: Key('search-history-remove-${entry.id}'),
        icon: const Icon(Icons.close),
        tooltip: 'Remove saved search',
        onPressed: () => onRemove(entry.id),
      ),
      onTap: () => onReplay(entry),
    );
  }

  String _buildSubtitle(SearchHistory entry) {
    final parts = <String>[];
    if (entry.tags.isNotEmpty) {
      parts.add('Tags: ${entry.tags.join(', ')}');
    }
    parts.add('Sort: ${entry.sortBy}');
    parts.add('Logic: ${entry.filterLogic.toUpperCase()}');
    return parts.join(' • ');
  }
}
