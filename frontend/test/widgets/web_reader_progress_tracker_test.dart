import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/widgets/web_reader_progress_tracker.dart';

void main() {
  group('parseProgressMessage', () {
    test('returns chapter and progress from valid JSON', () {
      final result = parseProgressMessage(
        '{"chapter":"Chapter 1","progress":0.5}',
      );
      expect(result, isNotNull);
      expect(result!.$1, 'Chapter 1');
      expect(result.$2, closeTo(0.5, 0.0001));
    });

    test('handles null chapter', () {
      final result = parseProgressMessage(
        '{"chapter":null,"progress":0.25}',
      );
      expect(result, isNotNull);
      expect(result!.$1, isNull);
      expect(result.$2, closeTo(0.25, 0.0001));
    });

    test('clamps progress to [0..1]', () {
      final high = parseProgressMessage('{"chapter":null,"progress":1.5}');
      expect(high!.$2, closeTo(1.0, 0.0001));

      final low = parseProgressMessage('{"chapter":null,"progress":-0.1}');
      expect(low!.$2, closeTo(0.0, 0.0001));
    });

    test('returns null for malformed JSON', () {
      expect(parseProgressMessage('not json'), isNull);
      expect(parseProgressMessage(''), isNull);
      expect(parseProgressMessage('{"chapter":"x"}'), isNull); // missing progress
    });
  });

  group('WebReaderProgressTracker widget', () {
    testWidgets('renders manual dispatch form', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WebReaderProgressTracker(
              resourceId: 'wr-1',
              onSignal: (_) {},
            ),
          ),
        ),
      );

      expect(find.byKey(const Key('progress-url')), findsOneWidget);
      expect(find.byKey(const Key('progress-chapter')), findsOneWidget);
      expect(find.byKey(const Key('progress-value')), findsOneWidget);
      expect(find.byKey(const Key('progress-dispatch')), findsOneWidget);
    });

    testWidgets('onProgressUpdate is invoked when _invokeProgressUpdate called', (tester) async {
      String? receivedChapter;
      double? receivedProgress;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WebReaderProgressTracker(
              resourceId: 'wr-1',
              onSignal: (_) {},
              onProgressUpdate: (chapter, progress) {
                receivedChapter = chapter;
                receivedProgress = progress;
              },
            ),
          ),
        ),
      );

      final state = tester.state<WebReaderProgressTrackerState>(
        find.byType(WebReaderProgressTracker),
      );
      state.invokeProgressUpdate('{"chapter":"Ch 2","progress":0.75}');

      expect(receivedChapter, 'Ch 2');
      expect(receivedProgress, closeTo(0.75, 0.0001));
    });

    testWidgets('onProgressUpdate handles null chapter gracefully', (tester) async {
      String? receivedChapter = 'sentinel';
      double? receivedProgress;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WebReaderProgressTracker(
              resourceId: 'wr-1',
              onSignal: (_) {},
              onProgressUpdate: (chapter, progress) {
                receivedChapter = chapter;
                receivedProgress = progress;
              },
            ),
          ),
        ),
      );

      final state = tester.state<WebReaderProgressTrackerState>(
        find.byType(WebReaderProgressTracker),
      );
      state.invokeProgressUpdate('{"chapter":null,"progress":0.0}');

      expect(receivedChapter, isNull);
      expect(receivedProgress, closeTo(0.0, 0.0001));
    });

    testWidgets('ignores malformed JS message', (tester) async {
      var called = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WebReaderProgressTracker(
              resourceId: 'wr-1',
              onSignal: (_) {},
              onProgressUpdate: (_, __) => called = true,
            ),
          ),
        ),
      );

      final state = tester.state<WebReaderProgressTrackerState>(
        find.byType(WebReaderProgressTracker),
      );
      state.invokeProgressUpdate('garbage');

      expect(called, isFalse);
    });
  });
}
