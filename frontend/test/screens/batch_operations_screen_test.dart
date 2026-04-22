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
      await tester.drag(find.byType(ListView), const Offset(0, 500));
      await tester.pumpAndSettle();

      expect(find.text('Copy failed'), findsOneWidget);
      expect(find.text('Local error: copy exploded'), findsOneWidget);
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
