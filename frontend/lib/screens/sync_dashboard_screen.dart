import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/sync/sync_bloc.dart';
import '../models/sync.dart';

class SyncDashboardScreen extends StatefulWidget {
  const SyncDashboardScreen({super.key});

  @override
  State<SyncDashboardScreen> createState() => _SyncDashboardScreenState();
}

class _SyncDashboardScreenState extends State<SyncDashboardScreen> {
  @override
  void initState() {
    super.initState();
    context.read<SyncBloc>().add(const LoadEcosystemStatus());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Sync Dashboard'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () =>
                context.read<SyncBloc>().add(const LoadEcosystemStatus()),
          ),
        ],
      ),
      body: BlocConsumer<SyncBloc, SyncState>(
        listener: (context, state) {
          if (state is SyncTriggered) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text(
                  'Sync triggered for ${state.job.platform} (job ${state.job.id.substring(0, 8)}…)',
                ),
              ),
            );
            context.read<SyncBloc>().add(const LoadEcosystemStatus());
          } else if (state is SyncError) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text('Error: ${state.failure}'),
                backgroundColor: Colors.red,
              ),
            );
          }
        },
        builder: (context, state) {
          if (state is SyncLoading) {
            return const Center(child: CircularProgressIndicator());
          }
          if (state is EcosystemStatusLoaded) {
            return _PlatformList(platforms: state.platforms);
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

class _PlatformList extends StatelessWidget {
  const _PlatformList({required this.platforms});

  final List<PlatformStatus> platforms;

  @override
  Widget build(BuildContext context) {
    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: platforms.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (context, index) {
        final p = platforms[index];
        return _PlatformCard(status: p);
      },
    );
  }
}

class _PlatformCard extends StatelessWidget {
  const _PlatformCard({required this.status});

  final PlatformStatus status;

  String _lastSyncText(SyncJob? job) {
    if (job == null) return 'Never synced';
    final ts = job.completedAt ?? job.createdAt;
    return 'Last: ${ts.toLocal().toString().substring(0, 16)} — ${job.itemsCreated} added';
  }

  Color _statusColor(SyncJobStatus? status) => switch (status) {
        SyncJobStatus.completed => Colors.green,
        SyncJobStatus.failed => Colors.red,
        SyncJobStatus.running => Colors.orange,
        _ => Colors.grey,
      };

  @override
  Widget build(BuildContext context) {
    final lastJob = status.lastSync;
    return Card(
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor:
              _statusColor(lastJob?.status).withOpacity(0.15),
          child: Icon(
            Icons.sync,
            color: _statusColor(lastJob?.status),
          ),
        ),
        title: Text(
          status.platform.toUpperCase(),
          style: const TextStyle(fontWeight: FontWeight.bold),
        ),
        subtitle: Text(_lastSyncText(lastJob)),
        trailing: ElevatedButton.icon(
          icon: const Icon(Icons.play_arrow, size: 16),
          label: const Text('Sync'),
          onPressed: () =>
              context.read<SyncBloc>().add(TriggerSync(status.platform)),
        ),
        onTap: () => Navigator.push(
          context,
          MaterialPageRoute<void>(
            builder: (_) => BlocProvider.value(
              value: context.read<SyncBloc>(),
              child: _PlatformSyncHistoryScreen(platform: status.platform),
            ),
          ),
        ),
      ),
    );
  }
}

class _PlatformSyncHistoryScreen extends StatefulWidget {
  const _PlatformSyncHistoryScreen({required this.platform});

  final String platform;

  @override
  State<_PlatformSyncHistoryScreen> createState() =>
      _PlatformSyncHistoryScreenState();
}

class _PlatformSyncHistoryScreenState
    extends State<_PlatformSyncHistoryScreen> {
  List<SyncJob>? _cachedJobs;

  @override
  void initState() {
    super.initState();
    context.read<SyncBloc>().add(LoadPlatformSyncs(widget.platform));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('${widget.platform.toUpperCase()} Sync History'),
      ),
      body: BlocListener<SyncBloc, SyncState>(
        listener: (context, state) {
          if (state is PlatformSyncsLoaded &&
              state.platform == widget.platform) {
            setState(() => _cachedJobs = state.jobs);
          }
        },
        child: Builder(
          builder: (context) {
            final jobs = _cachedJobs;
            if (jobs == null) {
              return const Center(child: CircularProgressIndicator());
            }
            if (jobs.isEmpty) {
              return const Center(child: Text('No sync history'));
            }
            return ListView.builder(
              itemCount: jobs.length,
              itemBuilder: (context, i) {
                final job = jobs[i];
                return ListTile(
                  leading: const Icon(Icons.history),
                  title: Text(
                    'Job ${job.id.substring(0, 8)}… — ${job.status.name}',
                  ),
                  subtitle: Text(
                    'Found: ${job.itemsFound}  Created: ${job.itemsCreated}  '
                    'Skipped: ${job.itemsSkipped}  Failed: ${job.itemsFailed}',
                  ),
                  trailing: Text(
                    job.createdAt.toLocal().toString().substring(0, 16),
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                );
              },
            );
          },
        ),
      ),
    );
  }
}
