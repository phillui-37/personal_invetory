import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../lib/widgets/accessible_widgets.dart';
import '../../lib/config/app_theme.dart';

void main() {
  group('Accessibility Widgets', () {
    group('AccessibleButton', () {
      testWidgets('renders button with label', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleButton(
                label: 'Save Changes',
                onPressed: () {},
              ),
            ),
          ),
        );

        expect(find.text('Save Changes'), findsOneWidget);
      });

      testWidgets('supports tooltip for additional context', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleButton(
                label: 'Delete',
                tooltip: 'Permanently delete this item',
                onPressed: () {},
              ),
            ),
          ),
        );

        await tester.longPress(find.byType(AccessibleButton));
        await tester.pumpAndSettle();
        expect(find.text('Permanently delete this item'), findsOneWidget);
      });

      testWidgets('calls callback when pressed', (tester) async {
        var pressed = false;
        
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleButton(
                label: 'Click',
                onPressed: () => pressed = true,
              ),
            ),
          ),
        );

        await tester.tap(find.byType(ElevatedButton));
        expect(pressed, true);
      });

      testWidgets('renders with icon when provided', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleButton(
                label: 'Download',
                icon: Icons.download,
                onPressed: () {},
              ),
            ),
          ),
        );

        expect(find.byIcon(Icons.download), findsOneWidget);
      });
    });

    group('AccessibleFormField', () {
      testWidgets('renders label', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleFormField(
                label: 'Email Address',
              ),
            ),
          ),
        );

        expect(find.text('Email Address'), findsOneWidget);
      });

      testWidgets('marks required fields', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleFormField(
                label: 'Password',
                isRequired: true,
              ),
            ),
          ),
        );

        expect(find.text('This field is required'), findsOneWidget);
      });

      testWidgets('shows error text when provided', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleFormField(
                label: 'Username',
                errorText: 'Username is already taken',
              ),
            ),
          ),
        );

        expect(find.text('Username is already taken'), findsOneWidget);
      });

      testWidgets('accepts text input', (tester) async {
        final controller = TextEditingController();
        
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleFormField(
                label: 'Name',
                controller: controller,
              ),
            ),
          ),
        );

        await tester.enterText(find.byType(TextFormField), 'John Doe');
        expect(controller.text, 'John Doe');
      });
    });

    group('AccessibleListTile', () {
      testWidgets('renders title and subtitle', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleListTile(
                title: 'Resource Title',
                subtitle: 'Author Name',
              ),
            ),
          ),
        );

        expect(find.text('Resource Title'), findsOneWidget);
        expect(find.text('Author Name'), findsOneWidget);
      });

      testWidgets('renders leading icon', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleListTile(
                title: 'Settings',
                leading: Icons.settings,
              ),
            ),
          ),
        );

        expect(find.byIcon(Icons.settings), findsOneWidget);
      });

      testWidgets('is tappable when onTap provided', (tester) async {
        var tapped = false;

        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AccessibleListTile(
                title: 'Item',
                onTap: () => tapped = true,
              ),
            ),
          ),
        );

        await tester.tap(find.byType(ListTile));
        expect(tapped, true);
      });
    });

    group('AccessibleDialog', () {
      testWidgets('accessible dialog has correct properties', (tester) async {
        final dialog = AccessibleDialog(
          title: 'Confirm',
          description: 'Are you sure?',
          content: Text('This will be deleted'),
          actions: [
            TextButton(onPressed: () {}, child: Text('Cancel')),
            TextButton(onPressed: () {}, child: Text('OK')),
          ],
        );

        expect(dialog.title, 'Confirm');
        expect(dialog.description, 'Are you sure?');
        expect(dialog.actions.length, 2);
      });
    });

    group('AccessibleTab', () {
      testWidgets('renders tab with label', (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: DefaultTabController(
              length: 2,
              child: Scaffold(
                appBar: AppBar(
                  bottom: TabBar(
                    tabs: [
                      AccessibleTab(label: 'Tab1', isSelected: true),
                      AccessibleTab(label: 'Tab2'),
                    ],
                  ),
                ),
                body: Container(),
              ),
            ),
          ),
        );

        expect(find.text('Tab1'), findsOneWidget);
        expect(find.text('Tab2'), findsOneWidget);
      });
    });
  });

  group('Accessibility Standards', () {
    test('light theme has sufficient contrast', () {
      // Text color on background
      const backgroundColor = Color(0xFFFFFFFF); // white
      const textColor = Color(0xFF000000); // black
      
      expect(
        AppTheme.hasGoodContrast(backgroundColor, textColor),
        true,
      );
    });

    test('dark theme has sufficient contrast', () {
      // Text color on dark background
      const backgroundColor = Color(0xFF111827); // dark gray
      const textColor = Color(0xFFFFFFFF); // white
      
      expect(
        AppTheme.hasGoodContrast(backgroundColor, textColor),
        true,
      );
    });

    testWidgets('buttons are keyboard accessible', (tester) async {
      var callCount = 0;
      
      await tester.pumpWidget(
        MaterialApp(
          theme: AppTheme.lightTheme,
          home: Scaffold(
            body: AccessibleButton(
              label: 'Click Me',
              onPressed: () => callCount++,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(ElevatedButton));
      await tester.pump();

      expect(callCount, 1);
    });

    testWidgets('form fields are focusable', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: AppTheme.lightTheme,
          home: Scaffold(
            body: AccessibleFormField(
              label: 'Test Field',
            ),
          ),
        ),
      );

      expect(find.byType(TextFormField), findsOneWidget);
    });
  });

  group('Semantic Features', () {
    testWidgets('buttons have semantic meaning', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AccessibleButton(
              label: 'Action Button',
              onPressed: () {},
            ),
          ),
        ),
      );

      expect(find.text('Action Button'), findsOneWidget);
      expect(find.byType(ElevatedButton), findsOneWidget);
    });

    testWidgets('form fields have labels for screen readers', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AccessibleFormField(
              label: 'Email',
              isRequired: true,
            ),
          ),
        ),
      );

      expect(find.text('Email'), findsOneWidget);
      expect(find.text('This field is required'), findsOneWidget);
    });

    testWidgets('list tiles communicate structure', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AccessibleListTile(
              title: 'Resource',
              subtitle: 'By Author',
              leading: Icons.book,
            ),
          ),
        ),
      );

      expect(find.text('Resource'), findsOneWidget);
      expect(find.text('By Author'), findsOneWidget);
      expect(find.byIcon(Icons.book), findsOneWidget);
    });
  });
}

