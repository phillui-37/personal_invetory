import 'package:flutter/material.dart';

import '../models/tag.dart';

class SearchFilterBar extends StatefulWidget {
  const SearchFilterBar({
    required this.tags,
    required this.selectedTags,
    required this.onTagsChanged,
    required this.onSortChanged,
    required this.onLogicChanged,
    this.sortOption = 'title',
    this.filterLogic = 'and',
    this.query = '',
    this.onQueryChanged,
    super.key,
  });

  final List<Tag> tags;
  final List<String> selectedTags;
  final ValueChanged<List<String>> onTagsChanged;
  final ValueChanged<String> onSortChanged;
  final ValueChanged<String> onLogicChanged;
  final String sortOption;
  final String filterLogic;
  final String query;
  final ValueChanged<String>? onQueryChanged;

  @override
  State<SearchFilterBar> createState() => _SearchFilterBarState();
}

class _SearchFilterBarState extends State<SearchFilterBar> {
  late List<String> _selectedTags;
  late String _sortOption;
  late String _filterLogic;
  late final TextEditingController _queryController;

  @override
  void initState() {
    super.initState();
    _selectedTags = List.from(widget.selectedTags);
    _sortOption = widget.sortOption;
    _filterLogic = widget.filterLogic;
    _queryController = TextEditingController(text: widget.query);
  }

  @override
  void didUpdateWidget(covariant SearchFilterBar oldWidget) {
    super.didUpdateWidget(oldWidget);

    if (!_sameTags(oldWidget.selectedTags, widget.selectedTags)) {
      _selectedTags = List<String>.from(widget.selectedTags);
    }
    if (oldWidget.sortOption != widget.sortOption) {
      _sortOption = widget.sortOption;
    }
    if (oldWidget.filterLogic != widget.filterLogic) {
      _filterLogic = widget.filterLogic;
    }
    if (_queryController.text != widget.query) {
      _queryController.value = TextEditingValue(
        text: widget.query,
        selection: TextSelection.collapsed(offset: widget.query.length),
      );
    }
  }

  @override
  void dispose() {
    _queryController.dispose();
    super.dispose();
  }

  void _toggleTag(String tagName) {
    setState(() {
      if (_selectedTags.contains(tagName)) {
        _selectedTags.remove(tagName);
      } else {
        _selectedTags.add(tagName);
      }
    });
    widget.onTagsChanged(_selectedTags);
  }

  bool _sameTags(List<String> left, List<String> right) {
    if (left.length != right.length) {
      return false;
    }
    for (var i = 0; i < left.length; i++) {
      if (left[i] != right[i]) {
        return false;
      }
    }
    return true;
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 12, 12, 8),
            child: TextField(
              key: const Key('search-query-input'),
              controller: _queryController,
              onChanged: widget.onQueryChanged,
              decoration: const InputDecoration(
                labelText: 'Search resources',
                border: OutlineInputBorder(),
                prefixIcon: Icon(Icons.search),
              ),
            ),
          ),
          if (_selectedTags.isNotEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              child: Row(
                children: [
                  Expanded(
                    child: DropdownButton<String>(
                      key: const Key('sort-dropdown'),
                      value: _sortOption,
                      isExpanded: true,
                      onChanged: (value) {
                        if (value != null) {
                          setState(() => _sortOption = value);
                          widget.onSortChanged(value);
                        }
                      },
                      items: const [
                        DropdownMenuItem(
                          value: 'title',
                          child: Text('Sort: Title'),
                        ),
                        DropdownMenuItem(
                          value: 'date_added',
                          child: Text('Sort: Date Added'),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 8),
                  DropdownButton<String>(
                    key: const Key('logic-dropdown'),
                    value: _filterLogic,
                    onChanged: (value) {
                      if (value != null) {
                        setState(() => _filterLogic = value);
                        widget.onLogicChanged(value);
                      }
                    },
                    items: const [
                      DropdownMenuItem(
                        value: 'and',
                        child: Text('AND'),
                      ),
                      DropdownMenuItem(
                        value: 'or',
                        child: Text('OR'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          if (widget.tags.isNotEmpty)
            SizedBox(
              height: 48,
              child: ListView(
                key: const Key('tag-chips-list'),
                scrollDirection: Axis.horizontal,
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                children: [
                  if (_selectedTags.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(right: 6),
                      child: InputChip(
                        key: const Key('clear-filters-chip'),
                        label: const Text('Clear'),
                        onDeleted: () {
                          setState(() => _selectedTags.clear());
                          widget.onTagsChanged(_selectedTags);
                        },
                        deleteIcon: const Icon(Icons.close, size: 16),
                      ),
                    ),
                  ...widget.tags.map(
                    (tag) => Padding(
                      padding: const EdgeInsets.only(right: 6),
                      child: FilterChip(
                        key: Key('filter-tag-${tag.name}'),
                        label: Text(tag.name),
                        selected: _selectedTags.contains(tag.name),
                        onSelected: (_) => _toggleTag(tag.name),
                      ),
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}
