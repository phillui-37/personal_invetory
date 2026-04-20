import 'package:flutter_test/flutter_test.dart';
import 'package:bloc_test/bloc_test.dart';

import '../../lib/blocs/onboarding/onboarding_bloc.dart';

void main() {
  group('OnboardingBloc', () {
    late OnboardingBloc onboardingBloc;

    setUp(() {
      onboardingBloc = OnboardingBloc();
    });

    tearDown(() {
      onboardingBloc.close();
    });

    group('LoadOnboardingStatus', () {
      blocTest<OnboardingBloc, OnboardingState>(
        'emits OnboardingInProgress with apiKey step',
        build: () => onboardingBloc,
        act: (bloc) => bloc.add(const LoadOnboardingStatus()),
        expect: () => [
          const OnboardingLoading(),
          const OnboardingInProgress(
            currentStep: OnboardingStep.apiKey,
            completedSteps: {},
          ),
        ],
      );
    });

    group('CompleteApiKeyStep', () {
      blocTest<OnboardingBloc, OnboardingState>(
        'moves to device registration step',
        build: () => onboardingBloc,
        seed: () => const OnboardingInProgress(
          currentStep: OnboardingStep.apiKey,
          completedSteps: {},
        ),
        act: (bloc) => bloc.add(const CompleteApiKeyStep('test-api-key')),
        expect: () => [
          const OnboardingLoading(),
          OnboardingInProgress(
            currentStep: OnboardingStep.deviceRegistration,
            completedSteps: {OnboardingStep.apiKey},
          ),
        ],
      );
    });

    group('CompleteDeviceStep', () {
      blocTest<OnboardingBloc, OnboardingState>(
        'moves to initial import step',
        build: () => onboardingBloc,
        seed: () => const OnboardingInProgress(
          currentStep: OnboardingStep.deviceRegistration,
          completedSteps: {OnboardingStep.apiKey},
        ),
        act: (bloc) => bloc.add(const CompleteDeviceStep('My Device')),
        expect: () => [
          const OnboardingLoading(),
          OnboardingInProgress(
            currentStep: OnboardingStep.initialImport,
            completedSteps: {OnboardingStep.apiKey, OnboardingStep.deviceRegistration},
          ),
        ],
      );
    });

    group('CompleteImportStep', () {
      blocTest<OnboardingBloc, OnboardingState>(
        'completes onboarding',
        build: () => onboardingBloc,
        seed: () => const OnboardingInProgress(
          currentStep: OnboardingStep.initialImport,
          completedSteps: {
            OnboardingStep.apiKey,
            OnboardingStep.deviceRegistration,
          },
        ),
        act: (bloc) => bloc.add(const CompleteImportStep()),
        expect: () => [
          const OnboardingLoading(),
          OnboardingInProgress(
            currentStep: OnboardingStep.complete,
            completedSteps: {
              OnboardingStep.apiKey,
              OnboardingStep.deviceRegistration,
              OnboardingStep.initialImport,
            },
          ),
        ],
      );
    });

    group('SkipOnboarding', () {
      blocTest<OnboardingBloc, OnboardingState>(
        'completes onboarding immediately',
        build: () => onboardingBloc,
        seed: () => const OnboardingInProgress(
          currentStep: OnboardingStep.apiKey,
          completedSteps: {},
        ),
        act: (bloc) => bloc.add(const SkipOnboarding()),
        expect: () => [const OnboardingComplete()],
      );
    });

    group('OnboardingStep enum', () {
      test('has correct next steps', () {
        expect(OnboardingStep.apiKey.next, OnboardingStep.deviceRegistration);
        expect(
          OnboardingStep.deviceRegistration.next,
          OnboardingStep.initialImport,
        );
        expect(OnboardingStep.initialImport.next, OnboardingStep.complete);
        expect(OnboardingStep.complete.next, null);
      });

      test('has correct previous steps', () {
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

      test('isFirst is true only for apiKey', () {
        expect(OnboardingStep.apiKey.isFirst, true);
        expect(OnboardingStep.deviceRegistration.isFirst, false);
        expect(OnboardingStep.initialImport.isFirst, false);
        expect(OnboardingStep.complete.isFirst, false);
      });
    });

    group('OnboardingInProgress', () {
      test('calculates progress correctly', () {
        const state1 = OnboardingInProgress(
          currentStep: OnboardingStep.apiKey,
          completedSteps: {},
        );
        expect(state1.progress, 1 / 4);

        const state2 = OnboardingInProgress(
          currentStep: OnboardingStep.deviceRegistration,
          completedSteps: {OnboardingStep.apiKey},
        );
        expect(state2.progress, 2 / 4);

        const state3 = OnboardingInProgress(
          currentStep: OnboardingStep.initialImport,
          completedSteps: {
            OnboardingStep.apiKey,
            OnboardingStep.deviceRegistration,
          },
        );
        expect(state3.progress, 3 / 4);
      });
    });
  });
}
