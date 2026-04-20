import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/onboarding/onboarding_bloc.dart';

class OnboardingScreen extends StatefulWidget {
  final VoidCallback onComplete;

  const OnboardingScreen({
    super.key,
    required this.onComplete,
  });

  @override
  State<OnboardingScreen> createState() => _OnboardingScreenState();
}

class _OnboardingScreenState extends State<OnboardingScreen> {
  final _apiKeyController = TextEditingController();
  final _deviceNameController = TextEditingController();

  @override
  void dispose() {
    _apiKeyController.dispose();
    _deviceNameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<OnboardingBloc, OnboardingState>(
      listener: (context, state) {
        if (state is OnboardingComplete) {
          widget.onComplete();
        }
      },
      child: Scaffold(
        body: BlocBuilder<OnboardingBloc, OnboardingState>(
          builder: (context, state) => SafeArea(
            child: switch (state) {
              OnboardingInitial() => const _LoadingView(),
              OnboardingLoading() => const _LoadingView(),
              OnboardingInProgress(currentStep: var step) =>
                _buildStepContent(context, step, state as OnboardingInProgress),
              OnboardingComplete() => const _CompleteView(),
            },
          ),
        ),
      ),
    );
  }

  Widget _buildStepContent(
    BuildContext context,
    OnboardingStep step,
    OnboardingInProgress state,
  ) {
    return Column(
      children: [
        _ProgressBar(progress: state.progress),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: switch (step) {
              OnboardingStep.apiKey => _ApiKeyStep(
                controller: _apiKeyController,
                onNext: () => context.read<OnboardingBloc>().add(
                  CompleteApiKeyStep(_apiKeyController.text),
                ),
              ),
              OnboardingStep.deviceRegistration => _DeviceStep(
                controller: _deviceNameController,
                onNext: () => context.read<OnboardingBloc>().add(
                  CompleteDeviceStep(_deviceNameController.text),
                ),
              ),
              OnboardingStep.initialImport => _ImportStep(
                onNext: () =>
                    context.read<OnboardingBloc>().add(const CompleteImportStep()),
                onSkip: () =>
                    context.read<OnboardingBloc>().add(const CompleteImportStep()),
              ),
              OnboardingStep.complete => const SizedBox.shrink(),
            },
          ),
        ),
      ],
    );
  }
}

class _ProgressBar extends StatelessWidget {
  final double progress;

  const _ProgressBar({required this.progress});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        LinearProgressIndicator(value: progress),
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            '${(progress * 100).toStringAsFixed(0)}% Complete',
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ),
      ],
    );
  }
}

class _LoadingView extends StatelessWidget {
  const _LoadingView();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const CircularProgressIndicator(),
          const SizedBox(height: 16),
          Text(
            'Setting up your inventory...',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ],
      ),
    );
  }
}

class _ApiKeyStep extends StatelessWidget {
  final TextEditingController controller;
  final VoidCallback onNext;

  const _ApiKeyStep({
    required this.controller,
    required this.onNext,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'API Key Setup',
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 8),
        Text(
          'Configure your API key for secure access to the inventory system.',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 24),
        Text(
          'API Key',
          style: Theme.of(context).textTheme.labelLarge,
        ),
        const SizedBox(height: 8),
        TextField(
          controller: controller,
          obscureText: true,
          decoration: InputDecoration(
            hintText: 'Enter your API key',
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
            ),
          ),
        ),
        const SizedBox(height: 24),
        SizedBox(
          width: double.infinity,
          child: ElevatedButton(
            onPressed: controller.text.isNotEmpty ? onNext : null,
            child: const Text('Continue'),
          ),
        ),
      ],
    );
  }
}

class _DeviceStep extends StatelessWidget {
  final TextEditingController controller;
  final VoidCallback onNext;

  const _DeviceStep({
    required this.controller,
    required this.onNext,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Device Registration',
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 8),
        Text(
          'Give this device a name for resource tracking.',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 24),
        Text(
          'Device Name',
          style: Theme.of(context).textTheme.labelLarge,
        ),
        const SizedBox(height: 8),
        TextField(
          controller: controller,
          decoration: InputDecoration(
            hintText: 'e.g., MacBook Pro, iPad',
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
            ),
          ),
        ),
        const SizedBox(height: 24),
        SizedBox(
          width: double.infinity,
          child: ElevatedButton(
            onPressed: controller.text.isNotEmpty ? onNext : null,
            child: const Text('Continue'),
          ),
        ),
      ],
    );
  }
}

class _ImportStep extends StatelessWidget {
  final VoidCallback onNext;
  final VoidCallback onSkip;

  const _ImportStep({
    required this.onNext,
    required this.onSkip,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Import Resources',
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 8),
        Text(
          'Import your existing resources to get started.',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 24),
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.primaryContainer,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'What to import?',
                style: Theme.of(context).textTheme.labelLarge,
              ),
              const SizedBox(height: 8),
              Text(
                '• Ebooks (PDF, EPUB, MOBI, AZW3)\n'
                '• Web Readers (URLs to track progress)\n'
                '• Images (PNG, JPG, GIF)\n'
                '• Videos (MP4, MKV)\n'
                '• Games (Steam, DLSite, Nintendo)',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ),
        ),
        const SizedBox(height: 24),
        SizedBox(
          width: double.infinity,
          child: ElevatedButton(
            onPressed: onNext,
            child: const Text('Start Importing'),
          ),
        ),
        const SizedBox(height: 12),
        SizedBox(
          width: double.infinity,
          child: OutlinedButton(
            onPressed: onSkip,
            child: const Text('Skip for Now'),
          ),
        ),
      ],
    );
  }
}

class _CompleteView extends StatelessWidget {
  const _CompleteView();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.check_circle,
            size: 80,
            color: Theme.of(context).colorScheme.primary,
          ),
          const SizedBox(height: 16),
          Text(
            'Onboarding Complete!',
            style: Theme.of(context).textTheme.headlineSmall,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 8),
          Text(
            'Your inventory system is ready to use.',
            style: Theme.of(context).textTheme.bodyMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 32),
          const CircularProgressIndicator(),
        ],
      ),
    );
  }
}
