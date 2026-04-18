import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/screens/bulk_import_screen.dart';

import '../support/fake_repositories.dart';

void main() {
  testWidgets('BulkImportScreen renders empty state', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: BulkImportScreen(
          ebookRepository: FakeEbookRepository(),
        ),
      ),
    );

    expect(find.text('Bulk Ebook Import'), findsOneWidget);
    expect(find.text('No files selected'), findsOneWidget);
  });

  testWidgets('BulkImportScreen disables Import All when no files are selected', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: BulkImportScreen(
          ebookRepository: FakeEbookRepository(),
        ),
      ),
    );

    final button = tester.widget<FilledButton>(
      find.byKey(const Key('bulk-import-submit')),
    );
    expect(button.onPressed, isNull);
  });

  testWidgets('BulkImportScreen loads directory entries through async lister', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: BulkImportScreen(
          ebookRepository: FakeEbookRepository(),
          pickDirectory: () async => '/books',
          listDirectoryFiles: (directory, recursive) async {
            expect(directory, '/books');
            expect(recursive, isFalse);
            return ['/books/a.epub', '/books/b.txt'];
          },
        ),
      ),
    );

    await tester.tap(find.byKey(const Key('bulk-import-pick-directory')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('bulk-item-/books/a.epub')), findsOneWidget);
    expect(find.byKey(const Key('bulk-item-/books/b.txt')), findsNothing);
  });
}
