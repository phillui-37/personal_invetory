import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

sealed class OnboardingEvent extends Equatable {
  const OnboardingEvent();

  @override
  List<Object?> get props => [];
}

final class LoadOnboardingStatus extends OnboardingEvent {
  const LoadOnboardingStatus();
}

final class CompleteApiKeyStep extends OnboardingEvent {
  const CompleteApiKeyStep(this.apiKey);

  final String apiKey;

  @override
  List<Object?> get props => [apiKey];
}

final class CompleteDeviceStep extends OnboardingEvent {
  const CompleteDeviceStep(this.deviceName);

  final String deviceName;

  @override
  List<Object?> get props => [deviceName];
}

final class CompleteImportStep extends OnboardingEvent {
  const CompleteImportStep();
}

final class SkipOnboarding extends OnboardingEvent {
  const SkipOnboarding();
}

sealed class OnboardingState extends Equatable {
  const OnboardingState();

  @override
  List<Object?> get props => [];
}

final class OnboardingInitial extends OnboardingState {
  const OnboardingInitial();
}

final class OnboardingLoading extends OnboardingState {
  const OnboardingLoading();
}

enum OnboardingStep {
  apiKey,
  deviceRegistration,
  initialImport,
  complete;

  bool get isFirst => this == OnboardingStep.apiKey;

  OnboardingStep? get next => switch (this) {
    apiKey => deviceRegistration,
    deviceRegistration => initialImport,
    initialImport => complete,
    complete => null,
  };

  OnboardingStep? get previous => switch (this) {
    apiKey => null,
    deviceRegistration => apiKey,
    initialImport => deviceRegistration,
    complete => initialImport,
  };

  String get displayName => switch (this) {
    apiKey => 'API Key Setup',
    deviceRegistration => 'Device Registration',
    initialImport => 'Import Resources',
    complete => 'Complete',
  };

  String get description => switch (this) {
    apiKey => 'Configure your API key for secure access',
    deviceRegistration => 'Register this device for resource tracking',
    initialImport => 'Import your existing resources (optional)',
    complete => 'Onboarding complete!',
  };
}

final class OnboardingInProgress extends OnboardingState {
  const OnboardingInProgress({
    required this.currentStep,
    required this.completedSteps,
  });

  final OnboardingStep currentStep;
  final Set<OnboardingStep> completedSteps;

  double get progress => (completedSteps.length + 1) / OnboardingStep.values.length;

  @override
  List<Object?> get props => [currentStep, completedSteps];
}

final class OnboardingComplete extends OnboardingState {
  const OnboardingComplete();
}

final class OnboardingBloc extends Bloc<OnboardingEvent, OnboardingState> {
  OnboardingBloc() : super(const OnboardingInitial()) {
    on<LoadOnboardingStatus>(_onLoadStatus);
    on<CompleteApiKeyStep>(_onCompleteApiKey);
    on<CompleteDeviceStep>(_onCompleteDevice);
    on<CompleteImportStep>(_onCompleteImport);
    on<SkipOnboarding>(_onSkipOnboarding);
  }

  Future<void> _onLoadStatus(LoadOnboardingStatus event, Emitter<OnboardingState> emit) async {
    emit(const OnboardingLoading());
    // Simulate checking onboarding status
    await Future.delayed(const Duration(milliseconds: 500));
    emit(
      const OnboardingInProgress(
        currentStep: OnboardingStep.apiKey,
        completedSteps: {},
      ),
    );
  }

  Future<void> _onCompleteApiKey(CompleteApiKeyStep event, Emitter<OnboardingState> emit) async {
    if (state is! OnboardingInProgress) return;
    final currentState = state as OnboardingInProgress;

    emit(const OnboardingLoading());
    // Simulate API key validation
    await Future.delayed(const Duration(milliseconds: 300));

    final nextStep = currentState.currentStep.next ?? OnboardingStep.complete;
    final completedSteps = {...currentState.completedSteps, currentState.currentStep};

    if (nextStep == OnboardingStep.complete) {
      emit(const OnboardingComplete());
    } else {
      emit(
        OnboardingInProgress(
          currentStep: nextStep,
          completedSteps: completedSteps,
        ),
      );
    }
  }

  Future<void> _onCompleteDevice(CompleteDeviceStep event, Emitter<OnboardingState> emit) async {
    if (state is! OnboardingInProgress) return;
    final currentState = state as OnboardingInProgress;

    emit(const OnboardingLoading());
    // Simulate device registration
    await Future.delayed(const Duration(milliseconds: 300));

    final nextStep = currentState.currentStep.next ?? OnboardingStep.complete;
    final completedSteps = {...currentState.completedSteps, currentState.currentStep};

    if (nextStep == OnboardingStep.complete) {
      emit(const OnboardingComplete());
    } else {
      emit(
        OnboardingInProgress(
          currentStep: nextStep,
          completedSteps: completedSteps,
        ),
      );
    }
  }

  Future<void> _onCompleteImport(CompleteImportStep event, Emitter<OnboardingState> emit) async {
    if (state is! OnboardingInProgress) return;
    final currentState = state as OnboardingInProgress;

    emit(const OnboardingLoading());
    // Simulate resource import
    await Future.delayed(const Duration(milliseconds: 300));

    final completedSteps = {...currentState.completedSteps, currentState.currentStep};
    emit(
      OnboardingInProgress(
        currentStep: OnboardingStep.complete,
        completedSteps: completedSteps,
      ),
    );
  }

  Future<void> _onSkipOnboarding(SkipOnboarding event, Emitter<OnboardingState> emit) async {
    emit(const OnboardingComplete());
  }
}
