import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/vault/vault_bloc.dart';
import '../models/vault.dart';

class VaultScreen extends StatelessWidget {
  const VaultScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Credential Vault')),
      body: BlocConsumer<VaultBloc, VaultState>(
        listener: (context, state) {
          if (state is VaultOperationSuccess) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('${state.operationType.name} successful')),
            );
            context.read<VaultBloc>().add(const CheckVaultStatus());
          }
        },
        builder: (context, state) {
          return Padding(
            padding: const EdgeInsets.all(16),
            child: switch (state) {
              VaultInitial() => _InitialView(),
              VaultLoading() =>
                const Center(child: CircularProgressIndicator()),
              VaultStatusLoaded(:final status) =>
                _StatusView(status: status),
              VaultPlatformsLoaded(:final platforms) =>
                _PlatformsView(platforms: platforms),
              VaultOperationSuccess() => const SizedBox.shrink(),
              VaultError(:final failure) =>
                Center(child: Text('Error: $failure')),
            },
          );
        },
      ),
    );
  }
}

class _InitialView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: ElevatedButton(
        onPressed: () =>
            context.read<VaultBloc>().add(const CheckVaultStatus()),
        child: const Text('Check Vault Status'),
      ),
    );
  }
}

class _StatusView extends StatefulWidget {
  const _StatusView({required this.status});
  final VaultStatus status;

  @override
  State<_StatusView> createState() => _StatusViewState();
}

class _StatusViewState extends State<_StatusView> {
  final _passwordController = TextEditingController();

  @override
  void dispose() {
    _passwordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.status.initialized) {
      return Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Text('Vault not initialized'),
          const SizedBox(height: 16),
          TextField(
            controller: _passwordController,
            obscureText: true,
            decoration: const InputDecoration(labelText: 'Master Password'),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => context.read<VaultBloc>().add(
                  InitializeVault(_passwordController.text),
                ),
            child: const Text('Initialize Vault'),
          ),
        ],
      );
    }

    if (!widget.status.unlocked) {
      return Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Text('Vault is locked'),
          const SizedBox(height: 16),
          TextField(
            controller: _passwordController,
            obscureText: true,
            decoration: const InputDecoration(labelText: 'Master Password'),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => context.read<VaultBloc>().add(
                  UnlockVault(_passwordController.text),
                ),
            child: const Text('Unlock'),
          ),
        ],
      );
    }

    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        const Icon(Icons.lock_open, size: 48, color: Colors.green),
        const SizedBox(height: 16),
        const Text('Vault is unlocked'),
        const SizedBox(height: 16),
        ElevatedButton(
          onPressed: () =>
              context.read<VaultBloc>().add(const LockVault()),
          child: const Text('Lock'),
        ),
        const SizedBox(height: 8),
        OutlinedButton(
          onPressed: () =>
              context.read<VaultBloc>().add(const LoadPlatforms()),
          child: const Text('View Platforms'),
        ),
      ],
    );
  }
}

class _PlatformsView extends StatelessWidget {
  const _PlatformsView({required this.platforms});
  final List<String> platforms;

  @override
  Widget build(BuildContext context) {
    if (platforms.isEmpty) {
      return const Center(child: Text('No platforms configured'));
    }
    return ListView.builder(
      itemCount: platforms.length,
      itemBuilder: (context, index) => ListTile(
        leading: const Icon(Icons.cloud),
        title: Text(platforms[index]),
      ),
    );
  }
}
