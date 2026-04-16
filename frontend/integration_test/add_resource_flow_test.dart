import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:personal_inventory_frontend/main.dart' as app;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('B10 add-resource flow', (tester) async {
    app.main();
    await tester.pumpAndSettle();

    expect(find.text('Resources'), findsOneWidget);

    await tester.tap(find.byKey(const Key('add-resource-fab')));
    await tester.pumpAndSettle();

    await tester.enterText(find.byKey(const Key('resource-title')), 'Integration Ebook');
    await tester.enterText(find.byKey(const Key('ebook-author')), 'Author Integration');
    await tester.enterText(find.byKey(const Key('ebook-file-format')), 'epub');
    await tester.enterText(find.byKey(const Key('location-device-id')), 'device-it');
    await tester.enterText(find.byKey(const Key('location-path')), '/integration/book.epub');
    await tester.tap(find.byKey(const Key('resource-submit')));
    await tester.pumpAndSettle();

    expect(find.text('Integration Ebook'), findsOneWidget);
  });
}
