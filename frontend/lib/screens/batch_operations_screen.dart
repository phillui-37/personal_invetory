import 'package:flutter/material.dart';

import '../models/batch_operations.dart';

class BatchOperationsScreen extends StatefulWidget {
  const BatchOperationsScreen({
    required this.onImport,
    required this.onUpdate,
    required this.onCopy,
    super.key,
  });

  final ValueChanged<BatchImportRequest> onImport;
  final ValueChanged<BatchMetadataUpdateRequest> onUpdate;
  final ValueChanged<BatchMetadataCopyRequest> onCopy;

  @override
  State<BatchOperationsScreen> createState() => _BatchOperationsScreenState();
}

class _BatchOperationsScreenState extends State<BatchOperationsScreen> {
  final _importPaths = TextEditingController();
  final _updateResourceIds = TextEditingController();
  final _updateFieldKey = TextEditingController();
  final _updateFieldValue = TextEditingController();
  final _copySourceId = TextEditingController();
  final _copyTargetIds = TextEditingController();
  bool _recursive = false;

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

  List<String> _csv(String input) {
    return input
        .split(',')
        .map((item) => item.trim())
        .where((item) => item.isNotEmpty)
        .toList();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Batch Operations')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const Text('Batch import'),
          TextField(
            key: const Key('batch-import-paths'),
            controller: _importPaths,
            decoration: const InputDecoration(
              labelText: 'Paths (comma separated)',
            ),
          ),
          SwitchListTile(
            value: _recursive,
            onChanged: (value) => setState(() => _recursive = value),
            title: const Text('Recursive'),
          ),
          ElevatedButton(
            key: const Key('batch-import-submit'),
            onPressed: () {
              widget.onImport(
                BatchImportRequest(paths: _csv(_importPaths.text), recursive: _recursive),
              );
            },
            child: const Text('Submit import request'),
          ),
          const Divider(height: 32),
          const Text('Batch metadata update'),
          TextField(
            key: const Key('batch-update-ids'),
            controller: _updateResourceIds,
            decoration: const InputDecoration(labelText: 'Resource IDs'),
          ),
          TextField(
            key: const Key('batch-update-field-key'),
            controller: _updateFieldKey,
            decoration: const InputDecoration(labelText: 'Field key'),
          ),
          TextField(
            key: const Key('batch-update-field-value'),
            controller: _updateFieldValue,
            decoration: const InputDecoration(labelText: 'Field value'),
          ),
          ElevatedButton(
            key: const Key('batch-update-submit'),
            onPressed: () {
              widget.onUpdate(
                BatchMetadataUpdateRequest(
                  resourceIds: _csv(_updateResourceIds.text),
                  fields: {
                    _updateFieldKey.text: _updateFieldValue.text,
                  },
                ),
              );
            },
            child: const Text('Submit metadata update request'),
          ),
          const Divider(height: 32),
          const Text('Batch metadata copy'),
          TextField(
            key: const Key('batch-copy-source-id'),
            controller: _copySourceId,
            decoration: const InputDecoration(labelText: 'Source resource ID'),
          ),
          TextField(
            key: const Key('batch-copy-target-ids'),
            controller: _copyTargetIds,
            decoration: const InputDecoration(labelText: 'Target IDs'),
          ),
          ElevatedButton(
            key: const Key('batch-copy-submit'),
            onPressed: () {
              widget.onCopy(
                BatchMetadataCopyRequest(
                  sourceResourceId: _copySourceId.text,
                  targetResourceIds: _csv(_copyTargetIds.text),
                ),
              );
            },
            child: const Text('Submit metadata copy request'),
          ),
        ],
      ),
    );
  }
}
