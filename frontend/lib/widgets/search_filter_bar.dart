import 'package:flutter/material.dart';

import '../models/tag.dart';

class SearchFilterBar extends StatefulWidget {
  const SearchFilterBar({
    required this.tags,
    required this.selectedTags,
    required this.onTagsChanged,
    required this.onSortChanged,
    required this.onLogicChanged,
    this.sortOption = 'date_added',
    this.filterLogic = 'and',
    super.key,
  });

  final List<Tag> tags;
  final List<String> selectedTags;
  final ValueChanged<List<String>> onTagsChanged;
  final ValueChanged<String> onSortChanged;
  final ValueChanged<String> onLogicChanged;
  final String sortOption;
  final String filterLogic;

  @override
  State<SearchFilterBar> createState() => _SearchFilterBarState();
}

class _SearchFilterBarState extends State<SearchFilterBar> {
  late List<String> _selectedTags;
  late String _sortOption;
  late String _filterLogic;

  @override
  void initState() {
    super.initState();
    _selectedTags = List.from(widget.selectedTags);
    _sortOption = widget.sortOption;
    _filterLogic = widget.filterLogic;
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

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Sort and filter logic controls
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
          // Tag filter chips
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
                        setState(() {
                          _selectedTags.clear();
                          _sortOption = 'date_added';
                          _filterLogic = 'and';
                        });
                        widget.onTagsChanged(_selectedTags);
                        widget.onSortChanged(_sortOption);
                        widget.onLogicChanged(_filterLogic);
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
