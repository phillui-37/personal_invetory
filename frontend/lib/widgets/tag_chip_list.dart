import 'package:flutter/material.dart';

import '../models/tag.dart';

class TagChipList extends StatefulWidget {
  const TagChipList({
    required this.tags,
    required this.onRemove,
    required this.onAdd,
    this.readOnly = false,
    super.key,
  });

  final List<Tag> tags;
  final void Function(String tagId) onRemove;
  final void Function(String name) onAdd;
  final bool readOnly;

  @override
  State<TagChipList> createState() => _TagChipListState();
}

class _TagChipListState extends State<TagChipList> {
  bool _addingTag = false;
  final _addController = TextEditingController();

  @override
  void dispose() {
    _addController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Wrap(
          spacing: 4,
          runSpacing: 4,
          children: [
            if (widget.tags.isEmpty && !_addingTag)
              const Chip(label: Text('No tags')),
            ...widget.tags.map(
              (tag) => Chip(
                label: Text(tag.name),
                deleteIcon: widget.readOnly
                    ? null
                    : Icon(
                        Icons.close,
                        key: Key('tag-chip-delete-${tag.id}'),
                        size: 16,
                      ),
                onDeleted: widget.readOnly
                    ? null
                    : () => widget.onRemove(tag.id),
              ),
            ),
            if (_addingTag)
              SizedBox(
                width: 160,
                child: TextField(
                  key: const Key('tag-add-input'),
                  controller: _addController,
                  autofocus: true,
                  decoration: const InputDecoration(
                    hintText: 'Tag name',
                    isDense: true,
                    contentPadding: EdgeInsets.symmetric(
                      horizontal: 8,
                      vertical: 6,
                    ),
                  ),
                  onSubmitted: (value) => _confirmAdd(value),
                ),
              ),
          ],
        ),
        if (!widget.readOnly)
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (!_addingTag)
                TextButton.icon(
                  key: const Key('tag-add-button'),
                  onPressed: () => setState(() {
                    _addingTag = true;
                    _addController.clear();
                  }),
                  icon: const Icon(Icons.add, size: 16),
                  label: const Text('Add tag'),
                ),
              if (_addingTag) ...[
                TextButton(
                  key: const Key('tag-add-confirm'),
                  onPressed: () => _confirmAdd(_addController.text),
                  child: const Text('Add'),
                ),
                TextButton(
                  onPressed: () => setState(() => _addingTag = false),
                  child: const Text('Cancel'),
                ),
              ],
            ],
          ),
      ],
    );
  }

  void _confirmAdd(String value) {
    final name = value.trim();
    if (name.isNotEmpty) {
      widget.onAdd(name);
    }
    setState(() {
      _addingTag = false;
      _addController.clear();
    });
  }
}
