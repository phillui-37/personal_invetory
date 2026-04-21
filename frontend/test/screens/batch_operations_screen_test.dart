import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/batch_operations.dart';
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
      await tester.pumpWidget(buildScreen());
      expect(find.text('Batch import'), findsOneWidget);
      expect(find.text('Batch metadata update'), findsOneWidget);
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
