import 'package:flutter/material.dart';

import '../models/repository_inputs.dart';

class WebReaderProgressTracker extends StatefulWidget {
  const WebReaderProgressTracker({
    required this.resourceId,
    required this.onSignal,
    super.key,
  });

  final String resourceId;
  final ValueChanged<WebReaderProgressSignal> onSignal;

  @override
  State<WebReaderProgressTracker> createState() => _WebReaderProgressTrackerState();
}

class _WebReaderProgressTrackerState extends State<WebReaderProgressTracker> {
  final _urlController = TextEditingController();
  final _chapterController = TextEditingController();
  final _progressController = TextEditingController();

  @override
  void dispose() {
    _urlController.dispose();
    _chapterController.dispose();
    _progressController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(top: 16),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('WebView progress signal'),
            TextField(
              key: const Key('progress-url'),
              controller: _urlController,
              decoration: const InputDecoration(labelText: 'Current URL'),
            ),
            TextField(
              key: const Key('progress-chapter'),
              controller: _chapterController,
              decoration: const InputDecoration(labelText: 'DOM chapter signal'),
            ),
            TextField(
              key: const Key('progress-value'),
              controller: _progressController,
              keyboardType: TextInputType.number,
              decoration: const InputDecoration(labelText: 'DOM progress [0..1]'),
            ),
            const SizedBox(height: 8),
            OutlinedButton(
              key: const Key('progress-dispatch'),
              onPressed: () {
                final progress = double.tryParse(_progressController.text);
                widget.onSignal(
                  WebReaderProgressSignal(
                    resourceId: widget.resourceId,
                    url: _urlController.text,
                    domChapter: _chapterController.text.isEmpty
                        ? null
                        : _chapterController.text,
                    domProgress: progress,
                  ),
                );
              },
              child: const Text('Dispatch progress signal'),
            ),
          ],
        ),
      ),
    );
  }
}
