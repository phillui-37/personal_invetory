import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:webview_flutter/webview_flutter.dart';

import '../models/repository_inputs.dart';

/// Parses a JSON message from the `ProgressSignal` JS channel.
/// Returns a (chapter, progress) record, or null on failure.
(String?, double)? parseProgressMessage(String json) {
  try {
    final map = jsonDecode(json) as Map<String, dynamic>;
    if (!map.containsKey('progress')) return null;
    final rawProgress = map['progress'];
    if (rawProgress is! num) return null;
    final progress = (rawProgress.toDouble()).clamp(0.0, 1.0);
    final chapter = map['chapter'] as String?;
    return (chapter, progress);
  } catch (_) {
    return null;
  }
}

class WebReaderProgressTracker extends StatefulWidget {
  const WebReaderProgressTracker({
    required this.resourceId,
    required this.onSignal,
    this.onProgressUpdate,
    super.key,
  });

  final String resourceId;
  final ValueChanged<WebReaderProgressSignal> onSignal;

  /// Called when the injected JS reports chapter/progress from the DOM.
  final void Function(String? chapter, double progress)? onProgressUpdate;

  @override
  State<WebReaderProgressTracker> createState() => WebReaderProgressTrackerState();
}

@visibleForTesting
class WebReaderProgressTrackerState extends State<WebReaderProgressTracker> {
  final _urlController = TextEditingController();
  final _chapterController = TextEditingController();
  final _progressController = TextEditingController();

  late final WebViewController? _webController;

  /// Exposed for testing: parse a raw JS channel message and fire [onProgressUpdate].
  void invokeProgressUpdate(String message) {
    final parsed = parseProgressMessage(message);
    if (parsed == null) return;
    widget.onProgressUpdate?.call(parsed.$1, parsed.$2);
  }

  @override
  void initState() {
    super.initState();
    try {
      _webController = WebViewController()
        ..setJavaScriptMode(JavaScriptMode.unrestricted)
        ..addJavaScriptChannel(
          'ProgressSignal',
          onMessageReceived: (msg) => invokeProgressUpdate(msg.message),
        )
        ..setNavigationDelegate(
          NavigationDelegate(
            onPageFinished: (_) async {
              final js = await rootBundle.loadString(
                'assets/frontend/js/progress_tracker.js',
              );
              await _webController!.runJavaScript(js);
            },
          ),
        );
    } catch (_) {
      // Platform channels unavailable (e.g., unit/widget tests without platform mock).
      _webController = null;
    }
  }

  @override
  void dispose() {
    _urlController.dispose();
    _chapterController.dispose();
    _progressController.dispose();
    super.dispose();
  }

  void _loadUrl() {
    final url = _urlController.text.trim();
    if (url.isNotEmpty) {
      _webController?.loadRequest(Uri.parse(url));
    }
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
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('progress-url'),
                    controller: _urlController,
                    decoration: const InputDecoration(labelText: 'Current URL'),
                    onSubmitted: (_) => _loadUrl(),
                  ),
                ),
                IconButton(
                  key: const Key('progress-load'),
                  icon: const Icon(Icons.open_in_browser),
                  tooltip: 'Load in WebView',
                  onPressed: _loadUrl,
                ),
              ],
            ),
            // TODO(D8): Replace with actual WebViewWidget once platform channels
            // are fully wired on all targets. The controller is already configured
            // with ProgressSignal channel + JS injection on page load.
            const SizedBox(height: 4),
            const Text(
              'Manual override:',
              style: TextStyle(fontSize: 12, color: Colors.grey),
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

