import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/batch/batch_bloc.dart';
import '../models/batch_operations.dart';
import '../widgets/app_failure_text.dart';
import '../widgets/loading_widgets.dart';

typedef BatchOperationCallback<T> = FutureOr<void> Function(T request);

class BatchOperationsScreen extends StatefulWidget {
  const BatchOperationsScreen({
    required this.onImport,
    required this.onUpdate,
    required this.onCopy,
    super.key,
  });

  final BatchOperationCallback<BatchImportRequest> onImport;
  final BatchOperationCallback<BatchMetadataUpdateRequest> onUpdate;
  final BatchOperationCallback<BatchMetadataCopyRequest> onCopy;

  @override
  State<BatchOperationsScreen> createState() => _BatchOperationsScreenState();
}

class _BatchOperationsScreenState extends State<BatchOperationsScreen> {
  final _importFormKey = GlobalKey<FormState>();
  final _updateFormKey = GlobalKey<FormState>();
  final _copyFormKey = GlobalKey<FormState>();
  final _importPaths = TextEditingController();
  final _updateResourceIds = TextEditingController();
  final _updateFieldKey = TextEditingController();
  final _updateFieldValue = TextEditingController();
  final _copySourceId = TextEditingController();
  final _copyTargetIds = TextEditingController();
  bool _recursive = false;
  BatchBloc? _batchBloc;
  _PendingSubmission? _pendingSubmission;
  _SubmissionFeedback? _feedback;

  @override
  void dispose() {
    _importPaths.dispose();
    _updateResourceIds.dispose();
    _updateFieldKey.dispose();
    _updateFieldValue.dispose();
    _copySourceId.dispose();
    _copyTargetIds.dispose();
    super.dispose();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    try {
      _batchBloc = BlocProvider.of<BatchBloc>(context, listen: false);
    } catch (_) {
      _batchBloc = null;
    }
  }

  List<String> _csv(String input) {
    return input
        .split(',')
        .map((item) => item.trim())
        .where((item) => item.isNotEmpty)
        .toList();
  }

  bool get _isBusy => _pendingSubmission != null;

  Future<void> _submitImport() async {
    if (!_importFormKey.currentState!.validate()) {
      return;
    }

    final paths = _csv(_importPaths.text);
    await _submitOperation(
      type: BatchOperationType.importResources,
      totalItems: paths.length,
      currentItemLabels: paths,
      fallbackSuccessTitle: 'Import request submitted',
      fallbackSuccessMessage: 'Queued ${paths.length} path(s) for import.',
      run: () => widget.onImport(
        BatchImportRequest(paths: paths, recursive: _recursive),
      ),
    );
  }

  Future<void> _submitUpdate() async {
    if (!_updateFormKey.currentState!.validate()) {
      return;
    }

    final resourceIds = _csv(_updateResourceIds.text);
    await _submitOperation(
      type: BatchOperationType.updateMetadata,
      totalItems: resourceIds.length,
      currentItemLabels: resourceIds,
      fallbackSuccessTitle: 'Update request submitted',
      fallbackSuccessMessage:
          'Queued ${resourceIds.length} resource(s) for update.',
      run: () => widget.onUpdate(
        BatchMetadataUpdateRequest(
          resourceIds: resourceIds,
          fields: {
            _updateFieldKey.text.trim(): _updateFieldValue.text.trim(),
          },
        ),
      ),
    );
  }

  Future<void> _submitCopy() async {
    if (!_copyFormKey.currentState!.validate()) {
      return;
    }

    final targetIds = _csv(_copyTargetIds.text);
    await _submitOperation(
      type: BatchOperationType.copyMetadata,
      totalItems: targetIds.length,
      currentItemLabels: targetIds,
      fallbackSuccessTitle: 'Copy request submitted',
      fallbackSuccessMessage:
          'Queued ${targetIds.length} target resource(s) for copy.',
      run: () => widget.onCopy(
        BatchMetadataCopyRequest(
          sourceResourceId: _copySourceId.text.trim(),
          targetResourceIds: targetIds,
        ),
      ),
    );
  }

  Future<void> _submitOperation({
    required BatchOperationType type,
    required int totalItems,
    required List<String> currentItemLabels,
    required String fallbackSuccessTitle,
    required String fallbackSuccessMessage,
    required FutureOr<void> Function() run,
  }) async {
    setState(() {
      _feedback = null;
      _pendingSubmission = _PendingSubmission(
        type: type,
        totalItems: totalItems,
        currentItemLabels: currentItemLabels.take(3).toList(),
      );
    });

    try {
      await Future<void>.sync(run);
      if (_batchBloc == null && mounted) {
        setState(() {
          _pendingSubmission = null;
          _feedback = _SubmissionFeedback.success(
            title: fallbackSuccessTitle,
            message: fallbackSuccessMessage,
          );
        });
      }
    } catch (error) {
      if (!mounted) {
        return;
      }
      setState(() {
        _pendingSubmission = null;
        _feedback = _SubmissionFeedback.error(
          title: '${type.label} failed',
          message: error.toString(),
        );
      });
    }
  }

  void _handleBatchState(BatchState state) {
    final pendingSubmission = _pendingSubmission;
    if (pendingSubmission == null) {
      return;
    }

    switch (state) {
      case BatchInitial():
      case BatchLoading():
        return;
      case BatchSuccess(:final response):
        if (response.type != pendingSubmission.type) {
          return;
        }
        setState(() {
          _pendingSubmission = null;
          _feedback = _feedbackFromResponse(response);
        });
      case BatchError(:final failure):
        setState(() {
          _pendingSubmission = null;
          _feedback = _SubmissionFeedback.error(
            title: '${pendingSubmission.type.label} failed',
            message: appFailureMessage(failure),
          );
        });
    }
  }

  _SubmissionFeedback _feedbackFromResponse(BatchOperationResponse response) {
    final successCount =
        response.results.where((result) => result.success).length;
    final failureCount = response.results.length - successCount;

    if (failureCount > 0) {
      return _SubmissionFeedback.info(
        title: '${response.type.label} completed with issues',
        message: '$successCount succeeded, $failureCount failed.',
      );
    }

    final message = successCount == 0
        ? '${response.type.label} request completed.'
        : '$successCount items succeeded.';

    return _SubmissionFeedback.success(
      title: '${response.type.label} completed',
      message: message,
    );
  }

  Widget _buildStatusPanel() {
    if (_pendingSubmission != null) {
      return Padding(
        padding: const EdgeInsets.only(bottom: 16),
        child: BatchOperationProgress(
          operationType: _pendingSubmission!.type.slug,
          itemsProcessed: 0,
          totalItems: _pendingSubmission!.totalItems,
          currentItemLabels: _pendingSubmission!.currentItemLabels,
        ),
      );
    }

    if (_feedback != null) {
      return Padding(
        padding: const EdgeInsets.only(bottom: 16),
        child: BatchFeedbackCard(
          title: _feedback!.title,
          message: _feedback!.message,
          tone: _feedback!.tone,
        ),
      );
    }

    return const SizedBox.shrink();
  }

  @override
  Widget build(BuildContext context) {
    final content = Scaffold(
      appBar: AppBar(title: const Text('Batch Operations')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _buildStatusPanel(),
          _SectionCard(
            title: 'Batch import',
            description: 'Paste one or more comma-separated paths to import.',
            child: Form(
              key: _importFormKey,
              autovalidateMode: AutovalidateMode.onUserInteraction,
              child: Column(
                children: [
                  TextFormField(
                    key: const Key('batch-import-paths'),
                    controller: _importPaths,
                    enabled: !_isBusy,
                    decoration: const InputDecoration(
                      labelText: 'Paths (comma separated)',
                      helperText: 'Example: /books/a.epub, /books/b.pdf',
                    ),
                    validator: (value) {
                      if (_csv(value ?? '').isEmpty) {
                        return 'Enter at least one path.';
                      }
                      return null;
                    },
                  ),
                  SwitchListTile(
                    value: _recursive,
                    onChanged: _isBusy
                        ? null
                        : (value) => setState(() => _recursive = value),
                    title: const Text('Recursive'),
                    contentPadding: EdgeInsets.zero,
                  ),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: ElevatedButton.icon(
                      key: const Key('batch-import-submit'),
                      onPressed: _isBusy ? null : _submitImport,
                      icon: const Icon(Icons.file_upload_outlined),
                      label: const Text('Submit import request'),
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          _SectionCard(
            title: 'Batch metadata update',
            description: 'Apply one metadata field to multiple resource IDs.',
            child: Form(
              key: _updateFormKey,
              autovalidateMode: AutovalidateMode.onUserInteraction,
              child: Column(
                children: [
                  TextFormField(
                    key: const Key('batch-update-ids'),
                    controller: _updateResourceIds,
                    enabled: !_isBusy,
                    decoration: const InputDecoration(
                      labelText: 'Resource IDs',
                      helperText: 'Separate multiple IDs with commas.',
                    ),
                    validator: (value) {
                      if (_csv(value ?? '').isEmpty) {
                        return 'Enter at least one resource ID.';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    key: const Key('batch-update-field-key'),
                    controller: _updateFieldKey,
                    enabled: !_isBusy,
                    decoration: const InputDecoration(labelText: 'Field key'),
                    validator: (value) {
                      if ((value ?? '').trim().isEmpty) {
                        return 'Enter a field key.';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    key: const Key('batch-update-field-value'),
                    controller: _updateFieldValue,
                    enabled: !_isBusy,
                    decoration: const InputDecoration(
                      labelText: 'Field value',
                      helperText:
                          'Use the value that should be applied to every resource.',
                    ),
                    validator: (value) {
                      if ((value ?? '').trim().isEmpty) {
                        return 'Enter a field value.';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 16),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: ElevatedButton.icon(
                      key: const Key('batch-update-submit'),
                      onPressed: _isBusy ? null : _submitUpdate,
                      icon: const Icon(Icons.edit_note),
                      label: const Text('Submit metadata update request'),
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          _SectionCard(
            title: 'Batch metadata copy',
            description:
                'Copy metadata from one source resource to many targets.',
            child: Form(
              key: _copyFormKey,
              autovalidateMode: AutovalidateMode.onUserInteraction,
              child: Column(
                children: [
                  TextFormField(
                    key: const Key('batch-copy-source-id'),
                    controller: _copySourceId,
                    enabled: !_isBusy,
                    decoration:
                        const InputDecoration(labelText: 'Source resource ID'),
                    validator: (value) {
                      if ((value ?? '').trim().isEmpty) {
                        return 'Enter a source resource ID.';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    key: const Key('batch-copy-target-ids'),
                    controller: _copyTargetIds,
                    enabled: !_isBusy,
                    decoration: const InputDecoration(
                      labelText: 'Target IDs',
                      helperText: 'Separate multiple target IDs with commas.',
                    ),
                    validator: (value) {
                      final targets = _csv(value ?? '');
                      if (targets.isEmpty) {
                        return 'Enter at least one target ID.';
                      }
                      final sourceId = _copySourceId.text.trim();
                      if (sourceId.isNotEmpty && targets.contains(sourceId)) {
                        return 'Source resource must not be in target IDs.';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 16),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: ElevatedButton.icon(
                      key: const Key('batch-copy-submit'),
                      onPressed: _isBusy ? null : _submitCopy,
                      icon: const Icon(Icons.copy_all_outlined),
                      label: const Text('Submit metadata copy request'),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );

    if (_batchBloc == null) {
      return content;
    }

    return BlocListener<BatchBloc, BatchState>(
      listener: (context, state) => _handleBatchState(state),
      child: content,
    );
  }
}

class _SectionCard extends StatelessWidget {
  const _SectionCard({
    required this.title,
    required this.description,
    required this.child,
  });

  final String title;
  final String description;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 4),
            Text(description, style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 16),
            child,
          ],
        ),
      ),
    );
  }
}

class _PendingSubmission {
  const _PendingSubmission({
    required this.type,
    required this.totalItems,
    required this.currentItemLabels,
  });

  final BatchOperationType type;
  final int totalItems;
  final List<String> currentItemLabels;
}

class _SubmissionFeedback {
  const _SubmissionFeedback({
    required this.title,
    required this.message,
    required this.tone,
  });

  factory _SubmissionFeedback.success({
    required String title,
    required String message,
  }) {
    return _SubmissionFeedback(
      title: title,
      message: message,
      tone: BatchFeedbackTone.success,
    );
  }

  factory _SubmissionFeedback.info({
    required String title,
    required String message,
  }) {
    return _SubmissionFeedback(
      title: title,
      message: message,
      tone: BatchFeedbackTone.info,
    );
  }

  factory _SubmissionFeedback.error({
    required String title,
    required String message,
  }) {
    return _SubmissionFeedback(
      title: title,
      message: message,
      tone: BatchFeedbackTone.error,
    );
  }

  final String title;
  final String message;
  final BatchFeedbackTone tone;
}

extension on BatchOperationType {
  String get label {
    return switch (this) {
      BatchOperationType.importResources => 'Import',
      BatchOperationType.updateMetadata => 'Update',
      BatchOperationType.copyMetadata => 'Copy',
    };
  }

  String get slug {
    return switch (this) {
      BatchOperationType.importResources => 'import',
      BatchOperationType.updateMetadata => 'update',
      BatchOperationType.copyMetadata => 'copy',
    };
  }
}
