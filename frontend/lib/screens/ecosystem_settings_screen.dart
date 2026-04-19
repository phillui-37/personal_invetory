import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/vault/vault_bloc.dart';

const _kSupportedPlatforms = [
  'steam',
  'dlsite',
  'fanza',
  'bookwalker',
  'kindle',
];

const _kPlatformLabels = {
  'steam': 'Steam',
  'dlsite': 'DLSite',
  'fanza': 'FANZA / DMM',
  'bookwalker': 'BookWalker',
  'kindle': 'Kindle (Amazon)',
};

const _kPlatformIcons = {
  'steam': Icons.videogame_asset,
  'dlsite': Icons.store,
  'fanza': Icons.movie,
  'bookwalker': Icons.book,
  'kindle': Icons.tablet_android,
};

class EcosystemSettingsScreen extends StatelessWidget {
  const EcosystemSettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Ecosystem Settings')),
      body: ListView.separated(
        padding: const EdgeInsets.all(16),
        itemCount: _kSupportedPlatforms.length,
        separatorBuilder: (_, __) => const SizedBox(height: 8),
        itemBuilder: (context, index) {
          final platform = _kSupportedPlatforms[index];
          return _PlatformSettingsTile(platform: platform);
        },
      ),
    );
  }
}

class _PlatformSettingsTile extends StatelessWidget {
  const _PlatformSettingsTile({required this.platform});

  final String platform;

  @override
  Widget build(BuildContext context) {
    final label = _kPlatformLabels[platform] ?? platform;
    final icon = _kPlatformIcons[platform] ?? Icons.cloud;
    return Card(
      child: ListTile(
        leading: Icon(icon),
        title: Text(label),
        subtitle: const Text('Tap to manage credentials'),
        trailing: const Icon(Icons.chevron_right),
        onTap: () => _showCredentialSheet(context, platform, label),
      ),
    );
  }

  void _showCredentialSheet(
      BuildContext context, String platform, String label) {
    showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      builder: (_) => BlocProvider.value(
        value: context.read<VaultBloc>(),
        child: _CredentialSheet(platform: platform, label: label),
      ),
    );
  }
}

class _CredentialSheet extends StatefulWidget {
  const _CredentialSheet({required this.platform, required this.label});

  final String platform;
  final String label;

  @override
  State<_CredentialSheet> createState() => _CredentialSheetState();
}

class _CredentialSheetState extends State<_CredentialSheet> {
  final _cookiesController = TextEditingController();
  bool _obscure = true;
  bool _saving = false;

  @override
  void dispose() {
    _cookiesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<VaultBloc, VaultState>(
      listener: (context, state) {
        if (!_saving) return;
        if (state is VaultOperationSuccess &&
            state.operationType == VaultOperationType.storeCredential) {
          _saving = false;
          Navigator.pop(context);
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('${widget.label} credentials saved to vault'),
            ),
          );
        } else if (state is VaultError) {
          _saving = false;
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Failed to save: ${state.failure}'),
              backgroundColor: Colors.red,
            ),
          );
        }
      },
      child: Padding(
        padding: EdgeInsets.only(
          left: 16,
          right: 16,
          top: 24,
          bottom: MediaQuery.of(context).viewInsets.bottom + 24,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '${widget.label} Credentials',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            const Text(
              'Paste JSON credentials (cookies / session data) exported from your browser.',
              style: TextStyle(color: Colors.grey),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: _cookiesController,
              maxLines: _obscure ? 1 : 5,
              obscureText: _obscure,
              decoration: InputDecoration(
                labelText: 'Credential JSON',
                border: const OutlineInputBorder(),
                suffixIcon: IconButton(
                  icon: Icon(_obscure ? Icons.visibility : Icons.visibility_off),
                  onPressed: () => setState(() => _obscure = !_obscure),
                ),
              ),
            ),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: const Text('Cancel'),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  onPressed: _saving
                      ? null
                      : () {
                          final credential = _cookiesController.text.trim();
                          if (credential.isEmpty) return;
                          setState(() => _saving = true);
                          context.read<VaultBloc>().add(
                                StoreCredential(
                                  platform: widget.platform,
                                  credentialType: 'session',
                                  plaintext: credential,
                                ),
                              );
                        },
                  child: _saving
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Save to Vault'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
