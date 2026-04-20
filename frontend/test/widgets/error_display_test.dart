import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../lib/models/failures.dart';
import '../../lib/widgets/error_display.dart';

void main() {
  group('Error Messaging', () {
    test('NetworkFailure generates retryable error message', () {
      const failure = NetworkFailure('Connection timeout');
      final message = getErrorMessage(failure);

      expect(message.title, 'Connection Problem');
      expect(message.isRetryable, true);
      expect(message.suggestion, isNotEmpty);
    });

    test('NotFoundFailure generates non-retryable error message', () {
      const failure = NotFoundFailure('ebook-123');
      final message = getErrorMessage(failure);

      expect(message.title, 'Resource Not Found');
      expect(message.isRetryable, false);
      expect(message.description, contains('ebook-123'));
    });

    test('ValidationFailure includes field and message', () {
      const failure = ValidationFailure(
        field: 'title',
        message: 'Title must not be empty',
      );
      final message = getErrorMessage(failure);

      expect(message.title, 'Invalid Input');
      expect(message.description, contains('title'));
      expect(message.description, contains('Title must not be empty'));
    });

    test('ServerFailure 401 provides vault guidance', () {
      const failure = ServerFailure(401);
      final message = getErrorMessage(failure);

      expect(message.title, 'Server Error');
      expect(message.description, contains('Authentication failed'));
      expect(message.suggestion, contains('Vault'));
      expect(message.isRetryable, false);
    });

    test('ServerFailure 429 is retryable', () {
      const failure = ServerFailure(429);
      final message = getErrorMessage(failure);

      expect(message.isRetryable, true);
      expect(message.description, contains('Too many requests'));
    });

    test('ServerFailure 500 is retryable', () {
      const failure = ServerFailure(500);
      final message = getErrorMessage(failure);

      expect(message.isRetryable, true);
    });

    test('LocalFailure with permission message is descriptive', () {
      const failure = LocalFailure('permission denied on /home/user/file');
      final message = getErrorMessage(failure);

      expect(message.title, 'Local Error');
      expect(message.description, contains('Permission denied'));
    });

    test('UnsupportedFailure provides guidance', () {
      const failure = UnsupportedFailure('file type .unknown not supported');
      final message = getErrorMessage(failure);

      expect(message.title, 'Not Supported');
      expect(message.description, contains('file type'));
    });
  });

  group('ErrorDisplay Widget', () {
    testWidgets('displays error title and description', (tester) async {
      const failure = NetworkFailure('Connection refused');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ErrorDisplay(failure: failure),
          ),
        ),
      );

      expect(find.text('Connection Problem'), findsOneWidget);
      expect(
        find.text('Unable to reach the server. Check your internet connection.'),
        findsOneWidget,
      );
    });

    testWidgets('shows retry button for retryable errors', (tester) async {
      const failure = ServerFailure(429);
      var retryCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ErrorDisplay(
              failure: failure,
              onRetry: () => retryCount++,
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.refresh), findsOneWidget);
      expect(find.text('Retry'), findsOneWidget);

      await tester.tap(find.text('Retry'));
      expect(retryCount, 1);
    });

    testWidgets('does not show retry button for non-retryable errors',
        (tester) async {
      const failure = NotFoundFailure('ebook-123');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ErrorDisplay(failure: failure),
          ),
        ),
      );

      expect(find.text('Retry'), findsNothing);
    });

    testWidgets('displays suggestion text when available', (tester) async {
      const failure = ValidationFailure(
        field: 'title',
        message: 'Title must not be empty',
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ErrorDisplay(failure: failure),
          ),
        ),
      );

      expect(find.text('Please check your input and try again.'),
          findsOneWidget);
    });

    testWidgets('shows error icon', (tester) async {
      const failure = NetworkFailure('timeout');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ErrorDisplay(failure: failure),
          ),
        ),
      );

      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });

    testWidgets('shows technical details when requested', (tester) async {
      const failure = LocalFailure('test error');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ErrorDisplay(
              failure: failure,
              showFullDetails: true,
            ),
          ),
        ),
      );

      expect(find.text('Technical details: LocalFailure'), findsOneWidget);
    });
  });

  group('CompactErrorDisplay Widget', () {
    testWidgets('displays compact error format', (tester) async {
      const failure = NetworkFailure('timeout');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CompactErrorDisplay(failure: failure),
          ),
        ),
      );

      expect(find.byIcon(Icons.error_outline), findsOneWidget);
      expect(find.text('Connection Problem'), findsOneWidget);
    });

    testWidgets('shows retry button in compact format', (tester) async {
      const failure = ServerFailure(500);
      var retryCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CompactErrorDisplay(
              failure: failure,
              onRetry: () => retryCount++,
            ),
          ),
        ),
      );

      expect(find.text('Retry'), findsOneWidget);
      await tester.tap(find.text('Retry'));
      expect(retryCount, 1);
    });
  });
}
