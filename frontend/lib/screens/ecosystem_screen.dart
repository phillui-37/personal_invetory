import 'package:flutter/material.dart';

import 'dedup_review_screen.dart';
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
        ],
      ),
    );
  }
}
