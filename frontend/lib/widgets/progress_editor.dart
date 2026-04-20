import 'package:flutter/material.dart';

import '../models/progress.dart';

class ProgressEditor extends StatefulWidget {
  const ProgressEditor({
    required this.initialProgress,
    required this.onSave,
    super.key,
  });

  final ResourceProgress? initialProgress;
  final void Function(double progress, String? notes) onSave;

  @override
  State<ProgressEditor> createState() => _ProgressEditorState();
}

class _ProgressEditorState extends State<ProgressEditor> {
  late double _value;
  late final TextEditingController _notesController;

  @override
  void initState() {
    super.initState();
    _value = widget.initialProgress?.progress ?? 0.0;
    _notesController =
        TextEditingController(text: widget.initialProgress?.notes ?? '');
  }

  @override
  void dispose() {
    _notesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(vertical: 8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  widget.initialProgress == null
                      ? 'No progress recorded'
                      : '${(_value * 100).round()}%',
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                IconButton(
                  key: const Key('progress-save'),
                  icon: const Icon(Icons.save),
                  tooltip: 'Save progress',
                  onPressed: () {
                    final notes = _notesController.text.trim().isEmpty
                        ? null
                        : _notesController.text.trim();
                    widget.onSave(_value, notes);
                  },
                ),
              ],
            ),
            Slider(
              key: const Key('progress-slider'),
              value: _value,
              min: 0.0,
              max: 1.0,
              divisions: 100,
              label: '${(_value * 100).round()}%',
              onChanged: (v) => setState(() => _value = v),
            ),
            TextField(
              key: const Key('progress-notes'),
              controller: _notesController,
              decoration: const InputDecoration(
                labelText: 'Notes (e.g. chapter)',
                isDense: true,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
