import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../lib/blocs/onboarding/onboarding_bloc.dart';
import '../../lib/screens/onboarding_screen.dart';

void main() {
  group('OnboardingScreen', () {
    late OnboardingBloc onboardingBloc;

    setUp(() {
      onboardingBloc = OnboardingBloc();
    });

    tearDown(() {
      onboardingBloc.close();
    });

    testWidgets('renders loading state initially', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>(
            create: (_) => onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('shows API key step after loading', (tester) async {
      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('API Key Setup'), findsOneWidget);
      expect(find.byType(TextField), findsWidgets);
    });

    testWidgets('API key step allows entering API key', (tester) async {
      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final textField = find.byType(TextField).first;
      await tester.enterText(textField, 'test-key-12345');
      expect(find.text('test-key-12345'), findsOneWidget);
    });

    testWidgets('continue button is disabled until API key entered', (tester) async {
      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.byType(ElevatedButton), findsOneWidget);
      final button = find.byType(ElevatedButton);
      expect(tester.widget<ElevatedButton>(button).onPressed, isNull);
    });

    testWidgets('continues to device step on API key completion', (tester) async {
      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final textField = find.byType(TextField).first;
      await tester.enterText(textField, 'test-key-12345');
      await tester.pumpAndSettle();

      final button = find.byType(ElevatedButton);
      await tester.tap(button);
      await tester.pumpAndSettle();

      expect(find.text('Device Registration'), findsOneWidget);
    });

    testWidgets('shows device registration step', (tester) async {
      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Go through API key step
      await tester.enterText(find.byType(TextField).first, 'api-key');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pumpAndSettle();

      expect(find.text('Device Registration'), findsOneWidget);
    });

    testWidgets('shows import resources step after device registration', (tester) async {
      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Go through API key step
      await tester.enterText(find.byType(TextField).first, 'api-key');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pumpAndSettle();

      // Go through device step
      await tester.enterText(find.byType(TextField).first, 'My Device');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pumpAndSettle();

      expect(find.text('Import Resources'), findsOneWidget);
    });

    testWidgets('import step shows skip option', (tester) async {
      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Navigate to import step
      await tester.enterText(find.byType(TextField).first, 'api-key');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField).first, 'My Device');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pumpAndSettle();

      expect(find.text('Skip for Now'), findsOneWidget);
    });

    testWidgets('calls onComplete when onboarding finishes', (tester) async {
      var completed = false;

      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(
              onComplete: () => completed = true,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Skip all steps
      onboardingBloc.add(const SkipOnboarding());
      await tester.pumpAndSettle();

      expect(completed, true);
    });

    testWidgets('shows progress bar', (tester) async {
      onboardingBloc.add(const LoadOnboardingStatus());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<OnboardingBloc>.value(
            value: onboardingBloc,
            child: OnboardingScreen(onComplete: () {}),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.byType(LinearProgressIndicator), findsOneWidget);
      expect(find.text('25% Complete'), findsOneWidget);
    });
  });
}
