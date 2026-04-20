import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/progress.dart';
import 'package:personal_inventory_frontend/widgets/progress_editor.dart';

void main() {
  group('ProgressEditor', () {
    testWidgets('shows no-progress text when initialProgress is null', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProgressEditor(
              initialProgress: null,
              onSave: (_, __) {},
            ),
          ),
        ),
      );

      expect(find.text('No progress recorded'), findsOneWidget);
      expect(find.byKey(const Key('progress-slider')), findsOneWidget);
    });

    testWidgets('shows current progress value when initialProgress provided', (tester) async {
      final progress = ResourceProgress(
        resourceId: 'r1',
        progress: 0.5,
        notes: 'Halfway',
        updatedAt: DateTime.utc(2025, 1, 1),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProgressEditor(
              initialProgress: progress,
              onSave: (_, __) {},
            ),
          ),
        ),
      );

      expect(find.text('50%'), findsOneWidget);
      expect(find.text('Halfway'), findsOneWidget);
    });

    testWidgets('calls onSave with slider value when save tapped', (tester) async {
      double? savedProgress;
      String? savedNotes;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProgressEditor(
              initialProgress: null,
              onSave: (progress, notes) {
                savedProgress = progress;
                savedNotes = notes;
              },
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('progress-save')));
      await tester.pump();

      expect(savedProgress, isNotNull);
      expect(savedNotes, isNull);
    });

    testWidgets('saves notes text entered in notes field', (tester) async {
      String? savedNotes;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ProgressEditor(
              initialProgress: null,
              onSave: (_, notes) => savedNotes = notes,
            ),
          ),
        ),
      );

      await tester.enterText(find.byKey(const Key('progress-notes')), 'Chapter 3');
      await tester.tap(find.byKey(const Key('progress-save')));
      await tester.pump();

      expect(savedNotes, 'Chapter 3');
    });
  });
}
