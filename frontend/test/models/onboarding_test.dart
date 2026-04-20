import 'package:flutter_test/flutter_test.dart';

import '../../lib/blocs/onboarding/onboarding_bloc.dart';

void main() {
  group('Onboarding Domain', () {
    group('OnboardingStep enum', () {
      test('apiKey is the first step', () {
        expect(OnboardingStep.apiKey.isFirst, true);
      });

      test('other steps are not first', () {
        expect(OnboardingStep.deviceRegistration.isFirst, false);
        expect(OnboardingStep.initialImport.isFirst, false);
        expect(OnboardingStep.complete.isFirst, false);
      });

      test('steps have correct next progression', () {
        expect(OnboardingStep.apiKey.next, OnboardingStep.deviceRegistration);
        expect(
          OnboardingStep.deviceRegistration.next,
          OnboardingStep.initialImport,
        );
        expect(OnboardingStep.initialImport.next, OnboardingStep.complete);
        expect(OnboardingStep.complete.next, null);
      });

      test('steps have correct previous progression', () {
        expect(OnboardingStep.apiKey.previous, null);
        expect(OnboardingStep.deviceRegistration.previous, OnboardingStep.apiKey);
        expect(
          OnboardingStep.initialImport.previous,
          OnboardingStep.deviceRegistration,
        );
        expect(
          OnboardingStep.complete.previous,
          OnboardingStep.initialImport,
        );
      });

      test('all steps have display names', () {
        expect(OnboardingStep.apiKey.displayName, isNotEmpty);
        expect(OnboardingStep.deviceRegistration.displayName, isNotEmpty);
        expect(OnboardingStep.initialImport.displayName, isNotEmpty);
        expect(OnboardingStep.complete.displayName, isNotEmpty);
      });

      test('all steps have descriptions', () {
        expect(OnboardingStep.apiKey.description, isNotEmpty);
        expect(OnboardingStep.deviceRegistration.description, isNotEmpty);
        expect(OnboardingStep.initialImport.description, isNotEmpty);
        expect(OnboardingStep.complete.description, isNotEmpty);
      });
    });

    group('OnboardingInProgress', () {
      test('calculates progress as 25% for first step', () {
        const state = OnboardingInProgress(
          currentStep: OnboardingStep.apiKey,
          completedSteps: {},
        );
        expect(state.progress, 1 / 4);
      });

      test('calculates progress as 50% for second step', () {
        const state = OnboardingInProgress(
          currentStep: OnboardingStep.deviceRegistration,
          completedSteps: {OnboardingStep.apiKey},
        );
        expect(state.progress, 2 / 4);
      });

      test('calculates progress as 75% for third step', () {
        const state = OnboardingInProgress(
          currentStep: OnboardingStep.initialImport,
          completedSteps: {
            OnboardingStep.apiKey,
            OnboardingStep.deviceRegistration,
          },
        );
        expect(state.progress, 3 / 4);
      });

      test('calculates progress as 100% for complete step', () {
        const state = OnboardingInProgress(
          currentStep: OnboardingStep.complete,
          completedSteps: {
            OnboardingStep.apiKey,
            OnboardingStep.deviceRegistration,
            OnboardingStep.initialImport,
          },
        );
        expect(state.progress, 4 / 4);
      });

      test('supports equality comparison', () {
        const state1 = OnboardingInProgress(
          currentStep: OnboardingStep.apiKey,
          completedSteps: {},
        );
        const state2 = OnboardingInProgress(
          currentStep: OnboardingStep.apiKey,
          completedSteps: {},
        );
        expect(state1, equals(state2));
      });
    });

    group('OnboardingEvents', () {
      test('LoadOnboardingStatus can be created', () {
        const event = LoadOnboardingStatus();
        expect(event, isNotNull);
      });

      test('CompleteApiKeyStep stores API key', () {
        const event = CompleteApiKeyStep('test-key');
        expect(event.apiKey, 'test-key');
      });

      test('CompleteDeviceStep stores device name', () {
        const event = CompleteDeviceStep('My Device');
        expect(event.deviceName, 'My Device');
      });

      test('CompleteImportStep can be created', () {
        const event = CompleteImportStep();
        expect(event, isNotNull);
      });

      test('SkipOnboarding can be created', () {
        const event = SkipOnboarding();
        expect(event, isNotNull);
      });
    });

    group('OnboardingStates', () {
      test('OnboardingInitial is a valid state', () {
        const state = OnboardingInitial();
        expect(state, isA<OnboardingState>());
      });

      test('OnboardingLoading is a valid state', () {
        const state = OnboardingLoading();
        expect(state, isA<OnboardingState>());
      });

      test('OnboardingInProgress is a valid state', () {
        const state = OnboardingInProgress(
          currentStep: OnboardingStep.apiKey,
          completedSteps: {},
        );
        expect(state, isA<OnboardingState>());
      });

      test('OnboardingComplete is a valid state', () {
        const state = OnboardingComplete();
        expect(state, isA<OnboardingState>());
      });
    });
  });
}
