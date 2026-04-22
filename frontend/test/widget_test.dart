// This is a basic Flutter widget test.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package. For example, you can send tap and scroll
// gestures. You can also use WidgetTester to find child widgets in the widget
// tree, read text, and verify that the values of widget properties are correct.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:personal_inventory_frontend/main.dart';
import 'package:personal_inventory_frontend/services/search_history_service.dart';

void main() {
  testWidgets('PersonalInventoryApp renders without errors',
      (WidgetTester tester) async {
    // Build our app and trigger a frame.
    await tester.pumpWidget(
      PersonalInventoryApp(searchHistoryService: SearchHistoryService()),
    );

    // Verify that the app shell renders with navigation bar
    expect(find.byIcon(Icons.inventory_2), findsOneWidget);
    expect(find.text('Inventory'), findsOneWidget);
  });
}
