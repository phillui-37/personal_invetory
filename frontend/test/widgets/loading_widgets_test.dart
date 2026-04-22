import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../lib/widgets/loading_widgets.dart';

void main() {
  group('Loading Widgets', () {
    group('ShimmerLoading', () {
      testWidgets('displays child when not loading', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: ShimmerLoading(
                isLoading: false,
                child: Text('Loaded Content'),
              ),
            ),
          ),
        );

        expect(find.text('Loaded Content'), findsOneWidget);
      });

      testWidgets('shows shimmer effect when loading', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: ShimmerLoading(
                isLoading: true,
                child: Container(
                  height: 100,
                  color: Colors.grey,
                ),
              ),
            ),
          ),
        );

        expect(find.byType(ShimmerLoading), findsOneWidget);
      });

      testWidgets('stops animation when loading completes', (tester) async {
        var isLoading = true;

        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: StatefulBuilder(
                builder: (context, setState) => Column(
                  children: [
                    ShimmerLoading(
                      isLoading: isLoading,
                      child: Text('Content'),
                    ),
                    ElevatedButton(
                      onPressed: () => setState(() => isLoading = false),
                      child: Text('Done'),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );

        expect(find.text('Content'), findsOneWidget);
        await tester.tap(find.text('Done'));
        await tester.pump();

        expect(find.text('Content'), findsOneWidget);
      });
    });

    group('SkeletonListItem', () {
      testWidgets('renders skeleton when loading', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SkeletonListItem(isLoading: true),
            ),
          ),
        );

        expect(find.byType(ListTile), findsOneWidget);
        expect(find.byType(ShimmerLoading), findsOneWidget);
      });

      testWidgets('shows placeholders for title and subtitle', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SkeletonListItem(isLoading: true),
            ),
          ),
        );

        // Should have multiple containers for skeleton effect
        expect(find.byType(Container), findsWidgets);
      });
    });

    group('SkeletonCard', () {
      testWidgets('renders card skeleton with specified height',
          (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SkeletonCard(
                height: 150,
                isLoading: true,
              ),
            ),
          ),
        );

        expect(find.byType(Card), findsOneWidget);
        expect(find.byType(ShimmerLoading), findsOneWidget);
      });

      testWidgets('has correct height', (tester) async {
        const testHeight = 200.0;

        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SkeletonCard(
                height: testHeight,
                isLoading: true,
              ),
            ),
          ),
        );

        final finder = find.byType(Card);
        expect(finder, findsOneWidget);
      });
    });

    group('SkeletonText', () {
      testWidgets('renders specified number of lines', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SkeletonText(
                lines: 3,
                isLoading: true,
              ),
            ),
          ),
        );

        expect(find.byType(Container), findsWidgets);
      });

      testWidgets('displays last line at reduced width', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SkeletonText(
                lines: 2,
                isLoading: true,
              ),
            ),
          ),
        );

        expect(find.byType(SkeletonText), findsOneWidget);
      });
    });

    group('OperationProgress', () {
      testWidgets('displays operation name', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: OperationProgress(
                completed: 5,
                total: 10,
                operationName: 'Importing Resources',
              ),
            ),
          ),
        );

        expect(find.text('Importing Resources'), findsOneWidget);
      });

      testWidgets('shows progress count', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: OperationProgress(
                completed: 7,
                total: 20,
                operationName: 'Updating',
              ),
            ),
          ),
        );

        expect(find.text('7 / 20 items'), findsOneWidget);
      });

      testWidgets('displays current item when provided', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: OperationProgress(
                completed: 1,
                total: 5,
                operationName: 'Copying',
                currentItem: 'file_name.pdf',
              ),
            ),
          ),
        );

        expect(find.text('file_name.pdf'), findsOneWidget);
      });

      testWidgets('shows progress bar', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: OperationProgress(
                completed: 5,
                total: 10,
                operationName: 'Test',
              ),
            ),
          ),
        );

        expect(find.byType(LinearProgressIndicator), findsOneWidget);
      });
    });

    group('SkeletonLoadingPage', () {
      testWidgets('renders multiple skeleton items', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SkeletonLoadingPage(
                isLoading: true,
                itemCount: 3,
              ),
            ),
          ),
        );

        expect(find.byType(SkeletonListItem), findsWidgets);
      });

      testWidgets('hides when not loading', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SkeletonLoadingPage(
                isLoading: false,
                itemCount: 5,
              ),
            ),
          ),
        );

        expect(find.byType(ListView), findsNothing);
      });
    });

    group('LoadingDialog', () {
      testWidgets('displays message', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: LoadingDialog(
                message: 'Processing your request...',
              ),
            ),
          ),
        );

        expect(find.text('Processing your request...'), findsOneWidget);
      });

      testWidgets('displays sub-message when provided', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: LoadingDialog(
                message: 'Loading',
                subMessage: 'This may take a few moments',
              ),
            ),
          ),
        );

        expect(find.text('This may take a few moments'), findsOneWidget);
      });

      testWidgets('shows progress indicator', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: LoadingDialog(
                message: 'Working...',
              ),
            ),
          ),
        );

        expect(find.byType(CircularProgressIndicator), findsOneWidget);
      });
    });

    group('BatchOperationProgress', () {
      testWidgets('displays operation type', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: BatchOperationProgress(
                operationType: 'import',
                itemsProcessed: 0,
                totalItems: 10,
              ),
            ),
          ),
        );

        expect(find.text('Batch IMPORT'), findsOneWidget);
      });

      testWidgets('shows percentage complete', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: BatchOperationProgress(
                operationType: 'update',
                itemsProcessed: 5,
                totalItems: 10,
              ),
            ),
          ),
        );

        expect(find.text('50%'), findsOneWidget);
      });

      testWidgets('displays item count', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: BatchOperationProgress(
                operationType: 'copy',
                itemsProcessed: 3,
                totalItems: 8,
              ),
            ),
          ),
        );

        expect(find.text('3 / 8 items'), findsOneWidget);
      });

      testWidgets('shows current items being processed', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: BatchOperationProgress(
                operationType: 'import',
                itemsProcessed: 2,
                totalItems: 5,
                currentItemLabels: ['book1.pdf', 'book2.pdf', 'book3.pdf'],
              ),
            ),
          ),
        );

        expect(find.text('• book1.pdf'), findsOneWidget);
        expect(find.text('• book2.pdf'), findsOneWidget);
      });

      testWidgets('shows estimated time remaining', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: BatchOperationProgress(
                operationType: 'import',
                itemsProcessed: 2,
                totalItems: 10,
                estimatedSecondsRemaining: 45,
              ),
            ),
          ),
        );

        expect(find.text('Est. time: 45s'), findsOneWidget);
      });

      testWidgets('shows error indicator when hasErrors true', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: BatchOperationProgress(
                operationType: 'import',
                itemsProcessed: 5,
                totalItems: 10,
                hasErrors: true,
              ),
            ),
          ),
        );

        expect(find.text('Batch IMPORT'), findsOneWidget);
      });
    });

    group('BatchFeedbackCard', () {
      testWidgets('renders success feedback details', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: BatchFeedbackCard.success(
                title: 'Import completed',
                message: '2 items succeeded.',
              ),
            ),
          ),
        );

        expect(find.text('Import completed'), findsOneWidget);
        expect(find.text('2 items succeeded.'), findsOneWidget);
        expect(find.byIcon(Icons.check_circle), findsOneWidget);
      });

      testWidgets('renders error feedback details', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: BatchFeedbackCard.error(
                title: 'Copy failed',
                message: 'Local error: copy exploded',
              ),
            ),
          ),
        );

        expect(find.text('Copy failed'), findsOneWidget);
        expect(find.text('Local error: copy exploded'), findsOneWidget);
        expect(find.byIcon(Icons.error_outline), findsOneWidget);
      });
    });
  });

  group('Loading UX Integration', () {
    testWidgets('skeleton properly transitions to loaded content',
        (tester) async {
      bool isLoading = true;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) => Column(
                children: [
                  if (isLoading)
                    SkeletonListItem(isLoading: true)
                  else
                    ListTile(
                      title: Text('Real Item'),
                    ),
                  ElevatedButton(
                    onPressed: () => setState(() => isLoading = false),
                    child: Text('Load'),
                  ),
                ],
              ),
            ),
          ),
        ),
      );

      expect(find.byType(SkeletonListItem), findsOneWidget);

      await tester.tap(find.text('Load'));
      await tester.pump();

      expect(find.text('Real Item'), findsOneWidget);
    });
  });
}
