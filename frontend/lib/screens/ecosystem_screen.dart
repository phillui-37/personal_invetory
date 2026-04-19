import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/device/device_bloc.dart';
import '../blocs/sync/sync_bloc.dart';
import 'dedup_review_screen.dart';
import 'device_management_screen.dart';
import 'ecosystem_settings_screen.dart';
import 'sync_dashboard_screen.dart';
import 'vault_screen.dart';

class EcosystemScreen extends StatelessWidget {
  const EcosystemScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Ecosystem')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: ListTile(
              leading: const Icon(Icons.sync),
              title: const Text('Sync Dashboard'),
              subtitle: const Text('Trigger and monitor platform syncs'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.push(
                context,
                MaterialPageRoute<void>(
                  builder: (_) => BlocProvider.value(
                    value: context.read<SyncBloc>(),
                    child: const SyncDashboardScreen(),
                  ),
                ),
              ),
            ),
          ),
          Card(
            child: ListTile(
              leading: const Icon(Icons.settings),
              title: const Text('Ecosystem Settings'),
              subtitle: const Text('Configure per-platform credentials'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.push(
                context,
                MaterialPageRoute<void>(
                  builder: (_) => const EcosystemSettingsScreen(),
                ),
              ),
            ),
          ),
          Card(
            child: ListTile(
              leading: const Icon(Icons.lock),
              title: const Text('Credential Vault'),
              subtitle: const Text('Manage platform credentials'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.push(
                context,
                MaterialPageRoute<void>(
                  builder: (_) => const VaultScreen(),
                ),
              ),
            ),
          ),
          Card(
            child: ListTile(
              leading: const Icon(Icons.compare_arrows),
              title: const Text('Dedup Review'),
              subtitle: const Text('Review and resolve duplicates'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.push(
                context,
                MaterialPageRoute<void>(
                  builder: (_) => const DedupReviewScreen(),
                ),
              ),
            ),
          ),
          Card(
            child: ListTile(
              leading: const Icon(Icons.devices),
              title: const Text('Device Management'),
              subtitle: const Text('Manage registered devices and their locations'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.push(
                context,
                MaterialPageRoute<void>(
                  builder: (_) => BlocProvider.value(
                    value: context.read<DeviceBloc>(),
                    child: const DeviceManagementScreen(),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
