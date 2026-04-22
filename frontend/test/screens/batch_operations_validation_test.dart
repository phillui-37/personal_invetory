import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/batch_operations.dart';
import 'package:personal_inventory_frontend/screens/batch_operations_screen.dart';

void main() {
  group('BatchOperationsScreen validation', () {
    Widget buildScreen({
      void Function(BatchImportRequest)? onImport,
      void Function(BatchMetadataUpdateRequest)? onUpdate,
      void Function(BatchMetadataCopyRequest)? onCopy,
    }) {
      return MaterialApp(
        home: SizedBox(
          width: 400,
          height: 800,
          child: BatchOperationsScreen(
            onImport: onImport ?? (_) {},
            onUpdate: onUpdate ?? (_) {},
            onCopy: onCopy ?? (_) {},
          ),
        ),
      );
    }

    testWidgets('blocks empty import submissions with inline feedback',
        (tester) async {
      var importCalls = 0;

      await tester.pumpWidget(
        buildScreen(onImport: (_) => importCalls++),
      );

      await tester.tap(find.byKey(const Key('batch-import-submit')));
      await tester.pump();

      expect(find.text('Enter at least one path.'), findsOneWidget);
      expect(importCalls, 0);
    });

    testWidgets('blocks update submissions without resource IDs and field key',
        (tester) async {
      var updateCalls = 0;

      await tester.pumpWidget(
        buildScreen(onUpdate: (_) => updateCalls++),
      );

      await tester.ensureVisible(find.byKey(const Key('batch-update-submit')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('batch-update-submit')));
      await tester.pump();

      expect(find.text('Enter at least one resource ID.'), findsOneWidget);
      expect(find.text('Enter a field key.'), findsOneWidget);
      expect(updateCalls, 0);
    });

    testWidgets('blocks copy submissions when source is in target IDs',
        (tester) async {
      var copyCalls = 0;

      await tester.pumpWidget(
        buildScreen(onCopy: (_) => copyCalls++),
      );

      await tester.drag(find.byType(ListView), const Offset(0, -500));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('batch-copy-source-id')),
        'src1',
      );
      await tester.enterText(
        find.byKey(const Key('batch-copy-target-ids')),
        'src1,tgt2',
      );

      await tester.tap(find.byKey(const Key('batch-copy-submit')));
      await tester.pump();

      expect(
        find.text('Source resource must not be in target IDs.'),
        findsOneWidget,
      );
      expect(copyCalls, 0);
    });
  });
}
