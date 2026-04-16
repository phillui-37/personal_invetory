import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/ebook/ebook_bloc.dart';
import '../blocs/web_reader/web_reader_bloc.dart';
import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../widgets/app_failure_text.dart';

class AddResourceScreen extends StatefulWidget {
  const AddResourceScreen({
    this.initialResourceType = ResourceType.ebook,
    this.resourceId,
    super.key,
  });

  final ResourceType initialResourceType;
  final String? resourceId;

  @override
  State<AddResourceScreen> createState() => _AddResourceScreenState();
}

class _AddResourceScreenState extends State<AddResourceScreen> {
  final _titleController = TextEditingController();
  final _notesController = TextEditingController();
  final _authorController = TextEditingController();
  final _fileFormatController = TextEditingController();
  final _urlController = TextEditingController();
  final _siteNameController = TextEditingController();
  final _deviceIdController = TextEditingController();
  final _pathController = TextEditingController();
  ResourceType _resourceType = ResourceType.ebook;
  StorageType _storageType = StorageType.localFs;
  String? _fieldError;
  NewLocationInput? _pendingLocation;
  String? _pendingResourceId;
  bool _prefilled = false;

  @override
  void initState() {
    super.initState();
    _resourceType = widget.initialResourceType;
    if (widget.resourceId != null) {
      if (_resourceType == ResourceType.ebook) {
        context.read<EbookBloc>().add(LoadEbookDetail(widget.resourceId!));
      } else {
        context.read<WebReaderBloc>().add(LoadWebReaderDetail(widget.resourceId!));
      }
    }
  }

  @override
  void dispose() {
    _titleController.dispose();
    _notesController.dispose();
    _authorController.dispose();
    _fileFormatController.dispose();
    _urlController.dispose();
    _siteNameController.dispose();
    _deviceIdController.dispose();
    _pathController.dispose();
    super.dispose();
  }

  void _showFailure(AppFailure failure) {
    if (failure is ValidationFailure) {
      setState(() => _fieldError = '${failure.field}: ${failure.message}');
    }
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(appFailureMessage(failure))));
  }

  void _submit() {
    setState(() => _fieldError = null);
    final id = widget.resourceId ?? _newResourceId();
    final resource = Resource(
      id: id,
      title: _titleController.text,
      notes: _notesController.text.isEmpty ? null : _notesController.text,
      resourceType: _resourceType,
    );

    final location = NewLocationInput(
      deviceId: _deviceIdController.text,
      pathOrUrl: _pathController.text,
      storageType: _storageType,
    );
    _pendingResourceId = id;
    _pendingLocation = _hasLocationInput() ? location : null;

    if (_resourceType == ResourceType.ebook) {
      if (widget.resourceId != null) {
        context.read<EbookBloc>().add(
          UpdateEbook(
            id,
            UpdateEbookInput(
              title: _titleController.text,
              author: _authorController.text.isEmpty ? null : _authorController.text,
              fileFormat: _fileFormatController.text.isEmpty
                  ? null
                  : _fileFormatController.text,
            ),
          ),
        );
      } else {
        context.read<EbookBloc>().add(
          AddEbook(
            NewEbookInput(
              resource: resource,
              meta: EbookMeta(
                resourceId: id,
                author: _authorController.text.isEmpty
                    ? null
                    : _authorController.text,
                fileFormat: _fileFormatController.text.isEmpty
                    ? null
                    : _fileFormatController.text,
              ),
            ),
          ),
        );
      }
      return;
    }

    if (widget.resourceId != null) {
      context.read<WebReaderBloc>().add(
        UpdateWebReader(
          id,
          UpdateWebReaderInput(
            title: _titleController.text,
            url: _urlController.text,
            siteName: _siteNameController.text.isEmpty ? null : _siteNameController.text,
          ),
        ),
      );
    } else {
      context.read<WebReaderBloc>().add(
        AddWebReader(
          NewWebReaderInput(
            resource: resource,
            meta: WebReaderMeta(
              resourceId: id,
              url: _urlController.text,
              siteName: _siteNameController.text.isEmpty
                  ? null
                  : _siteNameController.text,
            ),
          ),
        ),
      );
    }
  }

  String _newResourceId() {
    final timestamp = DateTime.now().microsecondsSinceEpoch;
    final entropy = Object().hashCode;
    return 'resource-$timestamp-$entropy';
  }

  void _onEbookOperationSuccess(EbookOperationSuccess state) {
    if (state.operationType == EbookOperationType.added ||
        state.operationType == EbookOperationType.updated) {
      final pendingResourceId = _pendingResourceId;
      final pendingLocation = _pendingLocation;
      if (pendingResourceId != null && pendingLocation != null) {
        context.read<EbookBloc>().add(AddEbookLocation(pendingResourceId, pendingLocation));
        return;
      }
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(widget.resourceId == null ? 'Ebook added' : 'Ebook updated')),
      );
      return;
    }

    if (state.operationType == EbookOperationType.locationAdded) {
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(widget.resourceId == null ? 'Ebook added' : 'Ebook updated')),
      );
    }
  }

  void _onWebReaderOperationSuccess(WebReaderOperationSuccess state) {
    if (state.operationType == WebReaderOperationType.added ||
        state.operationType == WebReaderOperationType.updated) {
      final pendingResourceId = _pendingResourceId;
      final pendingLocation = _pendingLocation;
      if (pendingResourceId != null && pendingLocation != null) {
        context.read<WebReaderBloc>().add(
          AddWebReaderLocation(pendingResourceId, pendingLocation),
        );
        return;
      }
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(widget.resourceId == null ? 'Web reader added' : 'Web reader updated'),
        ),
      );
      return;
    }

    if (state.operationType == WebReaderOperationType.locationAdded) {
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(widget.resourceId == null ? 'Web reader added' : 'Web reader updated'),
        ),
      );
    }
  }

  bool _hasLocationInput() {
    return _deviceIdController.text.trim().isNotEmpty && _pathController.text.trim().isNotEmpty;
  }

  void _prefillFromEbookDetail(EbookDetail detail) {
    if (_prefilled) {
      return;
    }
    _titleController.text = detail.resource.title;
    _notesController.text = detail.resource.notes ?? '';
    _authorController.text = detail.meta.author ?? '';
    _fileFormatController.text = detail.meta.fileFormat ?? '';
    if (detail.locations.isNotEmpty) {
      _deviceIdController.text = detail.locations.first.deviceId;
      _pathController.text = detail.locations.first.pathOrUrl;
      _storageType = detail.locations.first.storageType;
    }
    _prefilled = true;
    setState(() {});
  }

  void _prefillFromWebReaderDetail(WebReaderDetail detail) {
    if (_prefilled) {
      return;
    }
    _titleController.text = detail.resource.title;
    _notesController.text = detail.resource.notes ?? '';
    _urlController.text = detail.meta.url;
    _siteNameController.text = detail.meta.siteName ?? '';
    if (detail.locations.isNotEmpty) {
      _deviceIdController.text = detail.locations.first.deviceId;
      _pathController.text = detail.locations.first.pathOrUrl;
      _storageType = detail.locations.first.storageType;
    }
    _prefilled = true;
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return MultiBlocListener(
      listeners: [
        BlocListener<EbookBloc, EbookState>(
          listener: (context, state) {
            if (state is EbookDetailLoaded && widget.resourceId == state.ebook.resource.id) {
              _prefillFromEbookDetail(state.ebook);
            }
            if (state is EbookOperationSuccess) {
              _onEbookOperationSuccess(state);
            }
            if (state is EbookError) {
              _pendingResourceId = null;
              _pendingLocation = null;
              _showFailure(state.failure);
            }
          },
        ),
        BlocListener<WebReaderBloc, WebReaderState>(
          listener: (context, state) {
            if (state is WebReaderDetailLoaded &&
                widget.resourceId == state.webReader.resource.id) {
              _prefillFromWebReaderDetail(state.webReader);
            }
            if (state is WebReaderOperationSuccess) {
              _onWebReaderOperationSuccess(state);
            }
            if (state is WebReaderError) {
              _pendingResourceId = null;
              _pendingLocation = null;
              _showFailure(state.failure);
            }
          },
        ),
      ],
      child: Scaffold(
        appBar: AppBar(title: const Text('Add Resource')),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            DropdownButtonFormField<ResourceType>(
              key: const Key('resource-type-selector'),
              initialValue: _resourceType,
              decoration: const InputDecoration(labelText: 'Resource Type'),
              items: const [
                DropdownMenuItem(
                  value: ResourceType.ebook,
                  child: Text('Ebook'),
                ),
                DropdownMenuItem(
                  value: ResourceType.webReader,
                  child: Text('Web Reader'),
                ),
              ],
              onChanged: (value) {
                if (value != null) {
                  setState(() => _resourceType = value);
                }
              },
            ),
            TextField(
              key: const Key('resource-title'),
              controller: _titleController,
              decoration: InputDecoration(
                labelText: 'Title',
                errorText: _fieldError?.startsWith('title:') ?? false
                    ? _fieldError
                    : null,
              ),
            ),
            TextField(
              key: const Key('resource-notes'),
              controller: _notesController,
              decoration: const InputDecoration(labelText: 'Notes'),
            ),
            if (_resourceType == ResourceType.ebook) ...[
              TextField(
                key: const Key('ebook-author'),
                controller: _authorController,
                decoration: const InputDecoration(labelText: 'Author'),
              ),
              TextField(
                key: const Key('ebook-file-format'),
                controller: _fileFormatController,
                decoration: const InputDecoration(labelText: 'File format'),
              ),
            ] else ...[
              TextField(
                key: const Key('web-reader-url'),
                controller: _urlController,
                decoration: InputDecoration(
                  labelText: 'URL',
                  errorText: _fieldError?.startsWith('url:') ?? false
                      ? _fieldError
                      : null,
                ),
              ),
              TextField(
                key: const Key('web-reader-site-name'),
                controller: _siteNameController,
                decoration: const InputDecoration(labelText: 'Site Name'),
              ),
            ],
            const SizedBox(height: 12),
            const Text('Location'),
            TextField(
              key: const Key('location-device-id'),
              controller: _deviceIdController,
              decoration: const InputDecoration(labelText: 'Device ID'),
            ),
            TextField(
              key: const Key('location-path'),
              controller: _pathController,
              decoration: const InputDecoration(labelText: 'Path / URL'),
            ),
            DropdownButtonFormField<StorageType>(
              key: const Key('location-storage-type'),
              initialValue: _storageType,
              decoration: const InputDecoration(labelText: 'Storage Type'),
              items: StorageType.values
                  .map(
                    (type) => DropdownMenuItem(
                      value: type,
                      child: Text(type.name),
                    ),
                  )
                  .toList(),
              onChanged: (value) {
                if (value != null) {
                  setState(() => _storageType = value);
                }
              },
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              key: const Key('resource-submit'),
              onPressed: _submit,
              child: const Text('Submit'),
            ),
          ],
        ),
      ),
    );
  }
}
