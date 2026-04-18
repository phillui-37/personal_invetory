import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/ebook/ebook_bloc.dart';
import '../blocs/web_reader/web_reader_bloc.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../widgets/app_failure_text.dart';
import '../widgets/location_form_sheet.dart';
import '../widgets/web_reader_progress_tracker.dart';
import 'add_resource_screen.dart';

class ResourceDetailScreen extends StatefulWidget {
  const ResourceDetailScreen({
    required this.resourceId,
    required this.resourceType,
    super.key,
  });

  final String resourceId;
  final ResourceType resourceType;

  @override
  State<ResourceDetailScreen> createState() => _ResourceDetailScreenState();
}

class _ResourceDetailScreenState extends State<ResourceDetailScreen> {
  WebReaderDetail? _lastWebReaderDetail;
  List<ChapterCheck> _checkHistory = const [];

  @override
  void initState() {
    super.initState();
    _loadDetail();
  }

  void _loadDetail() {
    if (widget.resourceType == ResourceType.ebook) {
      context.read<EbookBloc>().add(LoadEbookDetail(widget.resourceId));
      return;
    }
    context.read<WebReaderBloc>().add(LoadWebReaderDetail(widget.resourceId));
    context.read<WebReaderBloc>().add(LoadCheckHistory(widget.resourceId));
  }

  void _confirmDelete() {
    final parentContext = context;
    showDialog<void>(
      context: parentContext,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Delete resource?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              Navigator.of(dialogContext).pop();
              if (widget.resourceType == ResourceType.ebook) {
                parentContext.read<EbookBloc>().add(DeleteEbook(widget.resourceId));
              } else {
                parentContext.read<WebReaderBloc>().add(
                  DeleteWebReader(widget.resourceId),
                );
              }
            },
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }

  void _addLocation() {
    showModalBottomSheet<void>(
      context: context,
      builder: (_) => LocationFormSheet(
        onSubmit: (input) {
          if (widget.resourceType == ResourceType.ebook) {
            context.read<EbookBloc>().add(AddEbookLocation(widget.resourceId, input));
          } else {
            context.read<WebReaderBloc>().add(
              AddWebReaderLocation(widget.resourceId, input),
            );
          }
        },
      ),
    );
  }

  void _removeLocation(String locationId) {
    final parentContext = context;
    showDialog<void>(
      context: parentContext,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Remove location?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              Navigator.of(dialogContext).pop();
              if (widget.resourceType == ResourceType.ebook) {
                parentContext.read<EbookBloc>().add(
                  RemoveEbookLocation(widget.resourceId, locationId),
                );
              } else {
                parentContext.read<WebReaderBloc>().add(
                  RemoveWebReaderLocation(widget.resourceId, locationId),
                );
              }
            },
            child: const Text('Remove'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return MultiBlocListener(
      listeners: [
        BlocListener<EbookBloc, EbookState>(
          listener: (context, state) {
            if (state is EbookOperationSuccess) {
              if (state.operationType == EbookOperationType.deleted) {
                Navigator.of(context).pop();
                return;
              }
              if (state.operationType == EbookOperationType.locationAdded ||
                  state.operationType == EbookOperationType.locationRemoved) {
                _loadDetail();
              }
            }
            if (state is EbookError) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text(appFailureMessage(state.failure))),
              );
            }
          },
        ),
        BlocListener<WebReaderBloc, WebReaderState>(
          listener: (context, state) {
            if (state is WebReaderDetailLoaded) {
              setState(() => _lastWebReaderDetail = state.webReader);
            }
            if (state is CheckHistoryLoaded) {
              setState(() => _checkHistory = state.history);
            }
            if (state is WebReaderOperationSuccess) {
              if (state.operationType == WebReaderOperationType.deleted) {
                Navigator.of(context).pop();
                return;
              }
              if (state.operationType == WebReaderOperationType.locationAdded ||
                  state.operationType == WebReaderOperationType.locationRemoved ||
                  state.operationType == WebReaderOperationType.progressTracked) {
                _loadDetail();
              }
            }
            if (state is ChapterCheckTriggered) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text(
                    state.result.hasNewChapter
                        ? 'New chapter: ${state.result.latestChapter ?? "available"}'
                        : 'No new chapter found.',
                  ),
                ),
              );
              context.read<WebReaderBloc>().add(
                LoadCheckHistory(widget.resourceId),
              );
            }
            if (state is WebReaderError) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text(appFailureMessage(state.failure))),
              );
            }
          },
        ),
      ],
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Resource Detail'),
          actions: [
            IconButton(
              icon: const Icon(Icons.edit),
              onPressed: () {
                Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => AddResourceScreen(
                      initialResourceType: widget.resourceType,
                      resourceId: widget.resourceId,
                    ),
                  ),
                );
              },
            ),
            IconButton(
              icon: const Icon(Icons.delete),
              onPressed: _confirmDelete,
            ),
          ],
        ),
        body: widget.resourceType == ResourceType.ebook
            ? BlocBuilder<EbookBloc, EbookState>(
                builder: (context, state) {
                  if (state is EbookLoading) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  if (state is EbookDetailLoaded) {
                    return _EbookDetailBody(
                      detail: state.ebook,
                      onAddLocation: _addLocation,
                      onRemoveLocation: _removeLocation,
                    );
                  }
                  return const Center(child: Text('No detail'));
                },
              )
            : BlocBuilder<WebReaderBloc, WebReaderState>(
                builder: (context, state) {
                  final detail = switch (state) {
                    WebReaderDetailLoaded(:final webReader) => webReader,
                    _ => _lastWebReaderDetail,
                  };
                  if (detail == null && state is WebReaderLoading) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  if (detail != null) {
                    return _WebReaderDetailBody(
                      detail: detail,
                      history: _checkHistory,
                      onAddLocation: _addLocation,
                      onRemoveLocation: _removeLocation,
                      onProgressSignal: (signal) {
                        context.read<WebReaderBloc>().add(
                          TrackWebReaderProgress(signal),
                        );
                      },
                      onCheckNow: () {
                        context.read<WebReaderBloc>().add(
                          TriggerChapterCheck(widget.resourceId),
                        );
                      },
                    );
                  }
                  return const Center(child: Text('No detail'));
                },
              ),
        floatingActionButton: FloatingActionButton.extended(
          key: const Key('add-location-button'),
          onPressed: _addLocation,
          icon: const Icon(Icons.add_location_alt),
          label: const Text('Add Location'),
        ),
      ),
    );
  }
}

class _EbookDetailBody extends StatelessWidget {
  const _EbookDetailBody({
    required this.detail,
    required this.onAddLocation,
    required this.onRemoveLocation,
  });

  final EbookDetail detail;
  final VoidCallback onAddLocation;
  final ValueChanged<String> onRemoveLocation;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(detail.resource.title, style: Theme.of(context).textTheme.titleLarge),
        Text('Author: ${detail.meta.author ?? '-'}'),
        Text('Format: ${detail.meta.fileFormat ?? '-'}'),
        const SizedBox(height: 12),
        const Text('Locations'),
        ...detail.locations.map(
          (location) => ListTile(
            title: Text(location.pathOrUrl),
            subtitle: Text(location.deviceId),
            trailing: Wrap(
              spacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                Chip(label: Text(location.storageType.name)),
                IconButton(
                  key: Key('remove-location-${location.id}'),
                  onPressed: () => onRemoveLocation(location.id),
                  icon: const Icon(Icons.delete_outline),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

class _WebReaderDetailBody extends StatelessWidget {
  const _WebReaderDetailBody({
    required this.detail,
    required this.history,
    required this.onAddLocation,
    required this.onRemoveLocation,
    required this.onProgressSignal,
    required this.onCheckNow,
  });

  final WebReaderDetail detail;
  final List<ChapterCheck> history;
  final VoidCallback onAddLocation;
  final ValueChanged<String> onRemoveLocation;
  final ValueChanged<WebReaderProgressSignal> onProgressSignal;
  final VoidCallback onCheckNow;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(detail.resource.title, style: Theme.of(context).textTheme.titleLarge),
        Text('URL: ${detail.meta.url}'),
        Text('Site: ${detail.meta.siteName ?? '-'}'),
        Text('Chapter: ${detail.meta.lastReadChapter ?? '-'}'),
        Text('Progress: ${detail.meta.progress?.toStringAsFixed(2) ?? '-'}'),
        const SizedBox(height: 12),
        const Text('Locations'),
        ...detail.locations.map(
          (location) => ListTile(
            title: Text(location.pathOrUrl),
            subtitle: Text(location.deviceId),
            trailing: Wrap(
              spacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                Chip(label: Text(location.storageType.name)),
                IconButton(
                  key: Key('remove-location-${location.id}'),
                  onPressed: () => onRemoveLocation(location.id),
                  icon: const Icon(Icons.delete_outline),
                ),
              ],
            ),
          ),
        ),
        WebReaderProgressTracker(
          resourceId: detail.resource.id,
          onSignal: onProgressSignal,
        ),
        const SizedBox(height: 12),
        _ChapterChecksSection(
          history: history,
          onCheckNow: onCheckNow,
        ),
      ],
    );
  }
}

class _ChapterChecksSection extends StatelessWidget {
  const _ChapterChecksSection({
    required this.history,
    required this.onCheckNow,
  });

  final List<ChapterCheck> history;
  final VoidCallback onCheckNow;

  @override
  Widget build(BuildContext context) {
    return ExpansionTile(
      title: const Text('Chapter Checks'),
      trailing: TextButton.icon(
        key: const Key('check-now-button'),
        onPressed: onCheckNow,
        icon: const Icon(Icons.refresh),
        label: const Text('Check Now'),
      ),
      children: history.isEmpty
          ? [const ListTile(title: Text('No check history.'))]
          : history.map(_buildCheckTile).toList(),
    );
  }

  Widget _buildCheckTile(ChapterCheck check) {
    return ListTile(
      key: Key('check-${check.id}'),
      leading: Icon(
        check.hasNewChapter ? Icons.new_releases : Icons.check_circle_outline,
        color: check.hasNewChapter ? Colors.green : Colors.grey,
      ),
      title: Text(
        check.latestChapter != null
            ? 'Latest: ${check.latestChapter}'
            : check.hasNewChapter
            ? 'New chapter available'
            : 'No new chapter',
      ),
      subtitle: Text(check.checkedAt.toLocal().toString()),
      trailing: check.errorMessage != null
          ? Tooltip(
              message: check.errorMessage!,
              child: const Icon(Icons.error_outline, color: Colors.red),
            )
          : null,
    );
  }
}
