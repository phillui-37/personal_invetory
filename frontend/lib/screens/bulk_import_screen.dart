import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';

import '../models/batch_operations.dart';
import '../models/result.dart';
import '../plugins/epub_metadata_extractor.dart';
import '../plugins/extractor_registry.dart';
import '../plugins/mobi_metadata_extractor.dart';
import '../plugins/pdf_metadata_extractor.dart';
import '../repositories/ebook_repository.dart';
import '../widgets/app_failure_text.dart';

enum BulkImportStatus {
  pending,
  extracting,
  extracted,
  failed,
}

final class _FileImportItem {
  const _FileImportItem({
    required this.path,
    required this.status,
    this.error,
  });

  final String path;
  final BulkImportStatus status;
  final String? error;

  _FileImportItem copyWith({
    BulkImportStatus? status,
    String? error,
  }) {
    return _FileImportItem(
      path: path,
      status: status ?? this.status,
      error: error,
    );
  }
}

class BulkImportScreen extends StatefulWidget {
  const BulkImportScreen({
    required this.ebookRepository,
    this.extractorRegistry,
    this.pickFiles,
    this.pickDirectory,
    this.listDirectoryFiles,
    super.key,
  });

  final EbookRepository ebookRepository;
  final ExtractorRegistry? extractorRegistry;
  final Future<List<String>> Function()? pickFiles;
  final Future<String?> Function()? pickDirectory;
  final Future<List<String>> Function(String directory, bool recursive)?
  listDirectoryFiles;

  @override
  State<BulkImportScreen> createState() => _BulkImportScreenState();
}

class _BulkImportScreenState extends State<BulkImportScreen> {
  late final ExtractorRegistry _extractorRegistry =
      widget.extractorRegistry ??
      ExtractorRegistry([
        PdfMetadataExtractor(),
        EpubMetadataExtractor(),
        MobiMetadataExtractor(),
      ]);

  final List<_FileImportItem> _items = [];
  bool _recursive = false;
  bool _isImporting = false;

  Future<void> _pickFiles() async {
    final picked = widget.pickFiles != null
        ? await widget.pickFiles!.call()
        : await _defaultPickFiles();
    if (picked.isEmpty) {
      return;
    }

    setState(() {
      _items
        ..clear()
        ..addAll(
          picked.map(
            (path) => _FileImportItem(path: path, status: BulkImportStatus.pending),
          ),
        );
    });
  }

  Future<void> _pickDirectory() async {
    final directory = widget.pickDirectory != null
        ? await widget.pickDirectory!.call()
        : await FilePicker.platform.getDirectoryPath();
    if (directory == null || directory.isEmpty) {
      return;
    }

    final files = widget.listDirectoryFiles != null
        ? await widget.listDirectoryFiles!(directory, _recursive)
        : await _defaultListDirectoryFiles(directory, _recursive);
    final supportedFiles = files.where(_isSupportedPath).toList();

    setState(() {
      _items
        ..clear()
        ..addAll(
          supportedFiles.map(
            (path) => _FileImportItem(path: path, status: BulkImportStatus.pending),
          ),
        );
    });
  }

  Future<List<String>> _defaultPickFiles() async {
    final result = await FilePicker.platform.pickFiles(
      allowMultiple: true,
      type: FileType.custom,
      allowedExtensions: const ['pdf', 'epub', 'mobi', 'azw3'],
    );
    return result?.files.map((file) => file.path).whereType<String>().toList() ?? const [];
  }

  Future<List<String>> _defaultListDirectoryFiles(
    String directory,
    bool recursive,
  ) async {
    final files = <String>[];
    await for (final entry in Directory(directory).list(recursive: recursive)) {
      if (entry is File && _isSupportedPath(entry.path)) {
        files.add(entry.path);
      }
    }
    return files;
  }

  Future<void> _importAll() async {
    setState(() => _isImporting = true);

    final entries = <BatchImportEntry>[];
    final pendingIndexes = <int>[];

    for (var i = 0; i < _items.length; i++) {
      final item = _items[i];
      setState(() {
        _items[i] = item.copyWith(status: BulkImportStatus.extracting, error: null);
      });

      final extractor = _extractorRegistry.forPath(item.path);
      if (extractor == null) {
        setState(() {
          _items[i] = item.copyWith(
            status: BulkImportStatus.failed,
            error: 'unsupported format',
          );
        });
        continue;
      }

      final result = await extractor.extract(item.path);
      switch (result) {
        case Success(:final value):
          entries.add(
            BatchImportEntry(
              title: value.title ?? _basenameWithoutExtension(item.path),
              author: value.author,
              isbn: value.isbn,
              publisher: value.publisher,
              language: value.language,
              fileFormat: value.fileFormat,
              filePath: item.path,
            ),
          );
          pendingIndexes.add(i);
          setState(() {
            _items[i] = item.copyWith(status: BulkImportStatus.extracted, error: null);
          });
        case Failure(:final failure):
          setState(() {
            _items[i] = item.copyWith(
              status: BulkImportStatus.failed,
              error: appFailureMessage(failure),
            );
          });
      }
    }

    if (entries.isNotEmpty) {
      final importResult = await widget.ebookRepository.batchImport(entries);
      if (importResult case Failure(:final failure)) {
        if (!mounted) {
          return;
        }
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(appFailureMessage(failure))),
        );
      } else if (importResult case Success(:final value)) {
        for (final failed in value.failed) {
          if (failed.index < pendingIndexes.length) {
            final itemIndex = pendingIndexes[failed.index];
            final current = _items[itemIndex];
            setState(() {
              _items[itemIndex] = current.copyWith(
                status: BulkImportStatus.failed,
                error: failed.error,
              );
            });
          }
        }
      }
    }

    if (mounted) {
      setState(() => _isImporting = false);
    }
  }

  bool _isSupportedPath(String path) =>
      const {'.pdf', '.epub', '.mobi', '.azw3'}.contains(_extension(path));

  String _extension(String path) {
    final dotIndex = path.lastIndexOf('.');
    if (dotIndex == -1) {
      return '';
    }
    return path.substring(dotIndex).toLowerCase();
  }

  String _basenameWithoutExtension(String path) {
    final name = path.split(RegExp(r'[/\\]')).last;
    final dotIndex = name.lastIndexOf('.');
    return dotIndex == -1 ? name : name.substring(0, dotIndex);
  }

  Color _statusColor(BulkImportStatus status) {
    return switch (status) {
      BulkImportStatus.pending => Colors.grey,
      BulkImportStatus.extracting => Colors.blue,
      BulkImportStatus.extracted => Colors.green,
      BulkImportStatus.failed => Colors.red,
    };
  }

  IconData _statusIcon(BulkImportStatus status) {
    return switch (status) {
      BulkImportStatus.pending => Icons.hourglass_empty,
      BulkImportStatus.extracting => Icons.sync,
      BulkImportStatus.extracted => Icons.check_circle,
      BulkImportStatus.failed => Icons.error,
    };
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Bulk Ebook Import')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              FilledButton.icon(
                key: const Key('bulk-import-pick-files'),
                onPressed: _pickFiles,
                icon: const Icon(Icons.file_open),
                label: const Text('Pick files'),
              ),
              OutlinedButton.icon(
                key: const Key('bulk-import-pick-directory'),
                onPressed: _pickDirectory,
                icon: const Icon(Icons.folder_open),
                label: const Text('Pick directory'),
              ),
            ],
          ),
          SwitchListTile(
            value: _recursive,
            onChanged: (value) => setState(() => _recursive = value),
            title: const Text('Recursive'),
          ),
          const SizedBox(height: 8),
          if (_items.isEmpty)
            const ListTile(title: Text('No files selected'))
          else
            ..._items.map(
              (item) => ListTile(
                key: Key('bulk-item-${item.path}'),
                leading: Icon(
                  _statusIcon(item.status),
                  color: _statusColor(item.status),
                ),
                title: Text(item.path.split(RegExp(r'[/\\]')).last),
                subtitle: item.error == null
                    ? Text(item.status.name)
                    : Text('${item.status.name}: ${item.error}'),
              ),
            ),
          const SizedBox(height: 12),
          FilledButton(
            key: const Key('bulk-import-submit'),
            onPressed: _items.isEmpty || _isImporting ? null : _importAll,
            child: _isImporting
                ? const SizedBox.square(
                    dimension: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('Import All'),
          ),
        ],
      ),
    );
  }
}
