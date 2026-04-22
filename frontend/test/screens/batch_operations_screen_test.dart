import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:personal_inventory_frontend/models/batch_operations.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/blocs/batch/batch_bloc.dart';
import 'package:personal_inventory_frontend/repositories/batch_operation_repository.dart';
import 'package:personal_inventory_frontend/screens/batch_operations_screen.dart';

void main() {
  group('BatchOperationsScreen', () {
    late List<BatchImportRequest> importCalls;
    late List<BatchMetadataUpdateRequest> updateCalls;
    late List<BatchMetadataCopyRequest> copyCalls;

    setUp(() {
      importCalls = [];
      updateCalls = [];
      copyCalls = [];
    });

    Widget buildScreen() {
      return MaterialApp(
        home: BatchOperationsScreen(
          onImport: (req) => importCalls.add(req),
          onUpdate: (req) => updateCalls.add(req),
          onCopy: (req) => copyCalls.add(req),
        ),
      );
    }

    Finder sectionCard(String title) {
      return find.ancestor(
        of: find.text(title),
        matching: find.byType(Card),
      );
    }

    testWidgets('renders all three sections', (tester) async {
      await tester.pumpWidget(
        SizedBox(
          width: 400,
          height: 800,
          child: buildScreen(),
        ),
      );
      expect(find.text('Batch import'), findsOneWidget);
      expect(find.text('Batch metadata update'), findsOneWidget);
      await tester.drag(find.byType(ListView), const Offset(0, -500));
      await tester.pumpAndSettle();
      expect(find.text('Batch metadata copy'), findsOneWidget);
    });

    testWidgets('calls onImport with entered paths', (tester) async {
      await tester.pumpWidget(buildScreen());
      await tester.enterText(
        find.byKey(const Key('batch-import-paths')),
        ' /a , , /b ',
      );
      await tester.tap(find.text('Recursive'));
      await tester.pump();
      await tester.tap(find.byKey(const Key('batch-import-submit')));
      await tester.pump();
      expect(importCalls, hasLength(1));
      expect(importCalls.first.paths, ['/a', '/b']);
      expect(importCalls.first.recursive, isTrue);
    });

    testWidgets('calls onUpdate with entered field key/value', (tester) async {
      await tester.pumpWidget(buildScreen());
      await tester.enterText(
        find.byKey(const Key('batch-update-ids')),
        ' id1, , id2 ',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-key')),
        'author',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-value')),
        'Jane',
      );
      await tester.ensureVisible(find.byKey(const Key('batch-update-submit')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('batch-update-submit')));
      await tester.pump();
      expect(updateCalls, hasLength(1));
      expect(updateCalls.first.resourceIds, ['id1', 'id2']);
      expect(updateCalls.first.fields, {'author': 'Jane'});
    });

    testWidgets('calls onCopy with entered source and target IDs',
        (tester) async {
      // Use a constrained test surface so the ListView can scroll.
      await tester.pumpWidget(
        SizedBox(
          width: 400,
          height: 800,
          child: buildScreen(),
        ),
      );
      // Scroll so the copy section is visible.
      await tester.drag(find.byType(ListView), const Offset(0, -500));
      await tester.pumpAndSettle();

      await tester.enterText(
        find.byKey(const Key('batch-copy-source-id')),
        'src1',
      );
      await tester.enterText(
        find.byKey(const Key('batch-copy-target-ids')),
        ' tgt1, , tgt2 ',
      );
      await tester.tap(find.byKey(const Key('batch-copy-submit')));
      await tester.pump();
      expect(copyCalls, hasLength(1));
      expect(copyCalls.first.sourceResourceId, 'src1');
      expect(copyCalls.first.targetResourceIds, ['tgt1', 'tgt2']);
    });

    testWidgets(
        'shows true overflow count when more than three import items are submitted',
        (tester) async {
      final completer = Completer<void>();

      await tester.pumpWidget(
        MaterialApp(
          home: BatchOperationsScreen(
            onImport: (_) => completer.future,
            onUpdate: (_) {},
            onCopy: (_) {},
          ),
        ),
      );

      await tester.enterText(
        find.byKey(const Key('batch-import-paths')),
        '/a,/b,/c,/d,/e',
      );
      await tester.tap(find.byKey(const Key('batch-import-submit')));
      await tester.pump();

      expect(find.text('• /a'), findsOneWidget);
      expect(find.text('• /b'), findsOneWidget);
      expect(find.text('• +3 more'), findsOneWidget);

      completer.complete();
      await tester.pumpAndSettle();
    });

    testWidgets(
        'shows busy progress and disables submit while import is running',
        (tester) async {
      final completer = Completer<void>();

      await tester.pumpWidget(
        MaterialApp(
          home: BatchOperationsScreen(
            onImport: (_) => completer.future,
            onUpdate: (_) {},
            onCopy: (_) {},
          ),
        ),
      );

      await tester.enterText(
        find.byKey(const Key('batch-import-paths')),
        '/a,/b',
      );
      await tester.tap(find.byKey(const Key('batch-import-submit')));
      await tester.pump();

      expect(find.text('Batch IMPORT'), findsOneWidget);
      expect(find.text('0 / 2 items'), findsOneWidget);

      final ElevatedButton button = tester.widget(
        find.byKey(const Key('batch-import-submit')),
      );
      expect(button.onPressed, isNull);

      completer.complete();
      await tester.pumpAndSettle();

      expect(find.text('Import request submitted'), findsOneWidget);
    });

    testWidgets('shows success feedback after batch import completes',
        (tester) async {
      final repository = _FakeBatchOperationRepository.importSuccess(
        const BatchOperationResponse(
          type: BatchOperationType.importResources,
          results: [
            BatchOperationItemResult(itemKey: '/a', success: true),
            BatchOperationItemResult(itemKey: '/b', success: true),
          ],
        ),
      );
      final batchBloc = BatchBloc(repository);

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider.value(
            value: batchBloc,
            child: BatchOperationsScreen(
              onImport: (req) => batchBloc.add(BatchImportRequested(req)),
              onUpdate: (_) {},
              onCopy: (_) {},
            ),
          ),
        ),
      );

      await tester.enterText(
        find.byKey(const Key('batch-import-paths')),
        '/a,/b',
      );
      await tester.tap(find.byKey(const Key('batch-import-submit')));
      await tester.pump();
      await tester.pump();

      expect(find.text('Import completed'), findsOneWidget);
      expect(find.text('2 items succeeded.'), findsOneWidget);
    });

    testWidgets('shows error feedback when batch import fully fails',
        (tester) async {
      final repository = _FakeBatchOperationRepository.importSuccess(
        const BatchOperationResponse(
          type: BatchOperationType.importResources,
          results: [
            BatchOperationItemResult(
              itemKey: '/a',
              success: false,
              errorMessage: 'bad file',
            ),
            BatchOperationItemResult(
              itemKey: '/b',
              success: false,
              errorMessage: 'missing metadata',
            ),
          ],
        ),
      );
      final batchBloc = BatchBloc(repository);

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider.value(
            value: batchBloc,
            child: BatchOperationsScreen(
              onImport: (req) => batchBloc.add(BatchImportRequested(req)),
              onUpdate: (_) {},
              onCopy: (_) {},
            ),
          ),
        ),
      );

      await tester.enterText(
        find.byKey(const Key('batch-import-paths')),
        '/a,/b',
      );
      await tester.tap(find.byKey(const Key('batch-import-submit')));
      await tester.pump();
      await tester.pump();

      final importCard = sectionCard('Batch import');
      expect(
        find.descendant(of: importCard, matching: find.text('Import failed')),
        findsOneWidget,
      );
      expect(
        find.descendant(
          of: importCard,
          matching: find.text('0 succeeded, 2 failed.'),
        ),
        findsOneWidget,
      );
      expect(
        find.descendant(
          of: importCard,
          matching: find.byIcon(Icons.error_outline),
        ),
        findsOneWidget,
      );
    });

    testWidgets(
        'shows update progress and completion feedback inside update section',
        (tester) async {
      final completer = Completer<void>();

      await tester.pumpWidget(
        SizedBox(
          width: 400,
          height: 800,
          child: MaterialApp(
            home: BatchOperationsScreen(
              onImport: (_) {},
              onUpdate: (_) => completer.future,
              onCopy: (_) {},
            ),
          ),
        ),
      );

      await tester.enterText(
        find.byKey(const Key('batch-update-ids')),
        'id1,id2',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-key')),
        'author',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-value')),
        'Jane',
      );
      await tester.ensureVisible(find.byKey(const Key('batch-update-submit')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('batch-update-submit')));
      await tester.pump();

      final updateCard = sectionCard('Batch metadata update');
      expect(
        find.descendant(of: updateCard, matching: find.text('Batch UPDATE')),
        findsOneWidget,
      );
      expect(
        find.descendant(of: updateCard, matching: find.text('0 / 2 items')),
        findsOneWidget,
      );

      final ElevatedButton button = tester.widget(
        find.byKey(const Key('batch-update-submit')),
      );
      expect(button.onPressed, isNull);

      completer.complete();
      await tester.pumpAndSettle();

      expect(
        find.descendant(
          of: updateCard,
          matching: find.text('Update request submitted'),
        ),
        findsOneWidget,
      );
    });

    testWidgets(
        'shows success feedback inside update section after bloc success',
        (tester) async {
      final repository = _FakeBatchOperationRepository.updateSuccess(
        const BatchOperationResponse(
          type: BatchOperationType.updateMetadata,
          results: [
            BatchOperationItemResult(itemKey: 'id1', success: true),
            BatchOperationItemResult(itemKey: 'id2', success: true),
          ],
        ),
      );
      final batchBloc = BatchBloc(repository);

      await tester.pumpWidget(
        SizedBox(
          width: 400,
          height: 800,
          child: MaterialApp(
            home: BlocProvider.value(
              value: batchBloc,
              child: BatchOperationsScreen(
                onImport: (_) {},
                onUpdate: (req) =>
                    batchBloc.add(BatchUpdateMetadataRequested(req)),
                onCopy: (_) {},
              ),
            ),
          ),
        ),
      );

      await tester.enterText(
        find.byKey(const Key('batch-update-ids')),
        'id1,id2',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-key')),
        'author',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-value')),
        'Jane',
      );
      await tester.ensureVisible(find.byKey(const Key('batch-update-submit')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('batch-update-submit')));
      await tester.pump();
      await tester.pump();

      final updateCard = sectionCard('Batch metadata update');
      expect(
        find.descendant(
            of: updateCard, matching: find.text('Update completed')),
        findsOneWidget,
      );
      expect(
        find.descendant(
            of: updateCard, matching: find.text('2 items succeeded.')),
        findsOneWidget,
      );
    });

    testWidgets(
        'shows failure feedback inside update section after bloc failure',
        (tester) async {
      final repository = _FakeBatchOperationRepository.updateFailure(
        const LocalFailure('update exploded'),
      );
      final batchBloc = BatchBloc(repository);

      await tester.pumpWidget(
        SizedBox(
          width: 400,
          height: 800,
          child: MaterialApp(
            home: BlocProvider.value(
              value: batchBloc,
              child: BatchOperationsScreen(
                onImport: (_) {},
                onUpdate: (req) =>
                    batchBloc.add(BatchUpdateMetadataRequested(req)),
                onCopy: (_) {},
              ),
            ),
          ),
        ),
      );

      await tester.enterText(
        find.byKey(const Key('batch-update-ids')),
        'id1,id2',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-key')),
        'author',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-value')),
        'Jane',
      );
      await tester.ensureVisible(find.byKey(const Key('batch-update-submit')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('batch-update-submit')));
      await tester.pump();
      await tester.pump();

      final updateCard = sectionCard('Batch metadata update');
      expect(
        find.descendant(of: updateCard, matching: find.text('Update failed')),
        findsOneWidget,
      );
      expect(
        find.descendant(
          of: updateCard,
          matching: find.text('Local error: update exploded'),
        ),
        findsOneWidget,
      );
    });

    testWidgets('shows failure feedback after batch copy fails',
        (tester) async {
      final repository = _FakeBatchOperationRepository.copyFailure(
        const LocalFailure('copy exploded'),
      );
      final batchBloc = BatchBloc(repository);

      await tester.pumpWidget(
        SizedBox(
          width: 400,
          height: 800,
          child: MaterialApp(
            home: BlocProvider.value(
              value: batchBloc,
              child: BatchOperationsScreen(
                onImport: (_) {},
                onUpdate: (_) {},
                onCopy: (req) => batchBloc.add(BatchCopyMetadataRequested(req)),
              ),
            ),
          ),
        ),
      );

      await tester.drag(find.byType(ListView), const Offset(0, -500));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('batch-copy-source-id')),
        'src1',
      );
      await tester.enterText(
        find.byKey(const Key('batch-copy-target-ids')),
        'tgt1,tgt2',
      );
      await tester.tap(find.byKey(const Key('batch-copy-submit')));
      await tester.pump();
      await tester.pump();
      final copyCard = sectionCard('Batch metadata copy');
      expect(
        find.descendant(of: copyCard, matching: find.text('Copy failed')),
        findsOneWidget,
      );
      expect(
        find.descendant(
          of: copyCard,
          matching: find.text('Local error: copy exploded'),
        ),
        findsOneWidget,
      );
    });

    testWidgets(
        'shows copy progress and completion feedback inside copy section',
        (tester) async {
      final completer = Completer<void>();

      await tester.pumpWidget(
        SizedBox(
          width: 400,
          height: 800,
          child: MaterialApp(
            home: BatchOperationsScreen(
              onImport: (_) {},
              onUpdate: (_) {},
              onCopy: (_) => completer.future,
            ),
          ),
        ),
      );

      await tester.drag(find.byType(ListView), const Offset(0, -500));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('batch-copy-source-id')),
        'src1',
      );
      await tester.enterText(
        find.byKey(const Key('batch-copy-target-ids')),
        'tgt1,tgt2',
      );
      await tester.tap(find.byKey(const Key('batch-copy-submit')));
      await tester.pump();

      final copyCard = sectionCard('Batch metadata copy');
      expect(
        find.descendant(of: copyCard, matching: find.text('Batch COPY')),
        findsOneWidget,
      );
      expect(
        find.descendant(of: copyCard, matching: find.text('0 / 2 items')),
        findsOneWidget,
      );

      final ElevatedButton button = tester.widget(
        find.byKey(const Key('batch-copy-submit')),
      );
      expect(button.onPressed, isNull);

      completer.complete();
      await tester.pumpAndSettle();

      expect(
        find.descendant(
          of: copyCard,
          matching: find.text('Copy request submitted'),
        ),
        findsOneWidget,
      );
    });

    testWidgets(
        'keeps callbacks untouched until the matching submit button is pressed',
        (tester) async {
      await tester.pumpWidget(
        SizedBox(
          width: 400,
          height: 800,
          child: buildScreen(),
        ),
      );
      await tester.enterText(
        find.byKey(const Key('batch-import-paths')),
        '/import-me',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-ids')),
        'id1',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-key')),
        'author',
      );
      await tester.enterText(
        find.byKey(const Key('batch-update-field-value')),
        'Jane',
      );
      await tester.drag(find.byType(ListView), const Offset(0, -500));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('batch-copy-source-id')),
        'src1',
      );

      expect(importCalls, isEmpty);
      expect(updateCalls, isEmpty);
      expect(copyCalls, isEmpty);

      await tester.ensureVisible(find.byKey(const Key('batch-update-submit')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('batch-update-submit')));
      await tester.pump();

      expect(importCalls, isEmpty);
      expect(updateCalls, hasLength(1));
      expect(copyCalls, isEmpty);
    });
  });
}

class _FakeBatchOperationRepository implements BatchOperationRepository {
  _FakeBatchOperationRepository({
    Result<BatchOperationResponse, AppFailure>? importResult,
    Result<BatchOperationResponse, AppFailure>? updateResult,
    Result<BatchOperationResponse, AppFailure>? copyResult,
  })  : _importResult = importResult,
        _updateResult = updateResult,
        _copyResult = copyResult;

  factory _FakeBatchOperationRepository.importSuccess(
    BatchOperationResponse response,
  ) {
    return _FakeBatchOperationRepository(
      importResult: Success<BatchOperationResponse, AppFailure>(response),
    );
  }

  factory _FakeBatchOperationRepository.updateSuccess(
    BatchOperationResponse response,
  ) {
    return _FakeBatchOperationRepository(
      updateResult: Success<BatchOperationResponse, AppFailure>(response),
    );
  }

  factory _FakeBatchOperationRepository.updateFailure(AppFailure failure) {
    return _FakeBatchOperationRepository(
      updateResult: Failure<BatchOperationResponse, AppFailure>(failure),
    );
  }

  factory _FakeBatchOperationRepository.copyFailure(AppFailure failure) {
    return _FakeBatchOperationRepository(
      copyResult: Failure<BatchOperationResponse, AppFailure>(failure),
    );
  }

  final Result<BatchOperationResponse, AppFailure>? _importResult;
  final Result<BatchOperationResponse, AppFailure>? _updateResult;
  final Result<BatchOperationResponse, AppFailure>? _copyResult;

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchCopyMetadata(
    BatchMetadataCopyRequest request,
  ) async {
    return _copyResult ??
        const Success<BatchOperationResponse, AppFailure>(
          BatchOperationResponse(
            type: BatchOperationType.copyMetadata,
            results: [],
          ),
        );
  }

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchImport(
    BatchImportRequest request,
  ) async {
    return _importResult ??
        const Success<BatchOperationResponse, AppFailure>(
          BatchOperationResponse(
            type: BatchOperationType.importResources,
            results: [],
          ),
        );
  }

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchUpdateMetadata(
    BatchMetadataUpdateRequest request,
  ) async {
    return _updateResult ??
        const Success<BatchOperationResponse, AppFailure>(
          BatchOperationResponse(
            type: BatchOperationType.updateMetadata,
            results: [],
          ),
        );
  }
}
