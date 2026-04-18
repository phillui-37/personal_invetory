import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/dedup/dedup_bloc.dart';
import '../models/dedup.dart';

class DedupReviewScreen extends StatelessWidget {
  const DedupReviewScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Dedup Review')),
      body: BlocConsumer<DedupBloc, DedupState>(
        listener: (context, state) {
          if (state is DedupOperationSuccess) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('${state.operationType.name} successful')),
            );
            context.read<DedupBloc>().add(const LoadWarnings());
          }
        },
        builder: (context, state) {
          return Padding(
            padding: const EdgeInsets.all(16),
            child: switch (state) {
              DedupInitial() => _InitialView(),
              DedupLoading() =>
                const Center(child: CircularProgressIndicator()),
              DedupWarningsLoaded(:final warnings) =>
                _WarningsView(warnings: warnings),
              DedupOperationSuccess() => const SizedBox.shrink(),
              DedupError(:final failure) =>
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
            context.read<DedupBloc>().add(const ScanDuplicates()),
        child: const Text('Scan for Duplicates'),
      ),
    );
  }
}

class _WarningsView extends StatelessWidget {
  const _WarningsView({required this.warnings});
  final List<DedupWarning> warnings;

  @override
  Widget build(BuildContext context) {
    if (warnings.isEmpty) {
      return const Center(child: Text('No duplicate warnings found'));
    }
    return ListView.builder(
      itemCount: warnings.length,
      itemBuilder: (context, index) => _WarningCard(warning: warnings[index]),
    );
  }
}

class _WarningCard extends StatelessWidget {
  const _WarningCard({required this.warning});
  final DedupWarning warning;

  @override
  Widget build(BuildContext context) {
    final percent = (warning.similarityScore * 100).round();
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  '$percent%',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(width: 8),
                const Text('similarity'),
              ],
            ),
            const SizedBox(height: 8),
            Text('Resource A: ${warning.resourceIdA}'),
            Text('Resource B: ${warning.resourceIdB}'),
            const SizedBox(height: 12),
            Row(
              children: [
                OutlinedButton(
                  onPressed: () => context.read<DedupBloc>().add(
                        DismissWarning(warning.id),
                      ),
                  child: const Text('Dismiss'),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  onPressed: () => context.read<DedupBloc>().add(
                        MergeResources(
                          warningId: warning.id,
                          keepId: warning.resourceIdA,
                          discardId: warning.resourceIdB,
                        ),
                      ),
                  child: const Text('Merge'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
