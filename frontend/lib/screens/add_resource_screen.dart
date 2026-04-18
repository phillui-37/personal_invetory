import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:file_picker/file_picker.dart';

import '../blocs/ebook/ebook_bloc.dart';
import '../blocs/game/game_bloc.dart';
import '../blocs/image/image_bloc.dart';
import '../blocs/video/video_bloc.dart';
import '../blocs/web_reader/web_reader_bloc.dart';
import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../plugins/epub_metadata_extractor.dart';
import '../plugins/extractor_registry.dart';
import '../plugins/metadata_extractor.dart';
import '../plugins/mobi_metadata_extractor.dart';
import '../plugins/pdf_metadata_extractor.dart';
import '../widgets/app_failure_text.dart';

class AddResourceScreen extends StatefulWidget {
  const AddResourceScreen({
    this.initialResourceType = ResourceType.ebook,
    this.resourceId,
    this.pickEbookFile,
    this.extractorRegistry,
    super.key,
  });

  final ResourceType initialResourceType;
  final String? resourceId;
  final Future<String?> Function()? pickEbookFile;
  final ExtractorRegistry? extractorRegistry;

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
  // Image fields
  final _widthController = TextEditingController();
  final _heightController = TextEditingController();
  final _fileSizeBytesController = TextEditingController();
  // Video fields
  final _durationController = TextEditingController();
  final _resolutionController = TextEditingController();
  // Game fields
  final _platformController = TextEditingController();
  final _storeController = TextEditingController();
  final _developerController = TextEditingController();
  final _publisherController = TextEditingController();
  final _manualNotesController = TextEditingController();

  ResourceType _resourceType = ResourceType.ebook;
  StorageType _storageType = StorageType.localFs;
  String? _fieldError;
  NewLocationInput? _pendingLocation;
  String? _pendingResourceId;
  bool _prefilled = false;

  late final ExtractorRegistry _extractorRegistry =
      widget.extractorRegistry ??
      ExtractorRegistry([
        PdfMetadataExtractor(),
        EpubMetadataExtractor(),
        MobiMetadataExtractor(),
      ]);

  @override
  void initState() {
    super.initState();
    _resourceType = widget.initialResourceType;
    if (widget.resourceId != null) {
      switch (_resourceType) {
        case ResourceType.ebook:
          context.read<EbookBloc>().add(LoadEbookDetail(widget.resourceId!));
        case ResourceType.webReader:
          context.read<WebReaderBloc>().add(LoadWebReaderDetail(widget.resourceId!));
        case ResourceType.image:
          context.read<ImageBloc>().add(LoadImageDetail(widget.resourceId!));
        case ResourceType.video:
          context.read<VideoBloc>().add(LoadVideoDetail(widget.resourceId!));
        case ResourceType.game:
          context.read<GameBloc>().add(LoadGameDetail(widget.resourceId!));
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
    _widthController.dispose();
    _heightController.dispose();
    _fileSizeBytesController.dispose();
    _durationController.dispose();
    _resolutionController.dispose();
    _platformController.dispose();
    _storeController.dispose();
    _developerController.dispose();
    _publisherController.dispose();
    _manualNotesController.dispose();
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

  Future<void> _pickAndExtractMetadata() async {
    final path = widget.pickEbookFile != null
        ? await widget.pickEbookFile!.call()
        : await _defaultPickEbookFile();
    if (path == null || path.isEmpty) {
      return;
    }

    _pathController.text = path;
    final extractor = _extractorRegistry.forPath(path);
    if (extractor == null) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Auto-fill not available for this format')),
      );
      return;
    }

    final result = await extractor.extract(path);
    if (!mounted) {
      return;
    }

    result.when(
      success: (meta) {
        _applyExtractedMeta(path, meta);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Metadata extracted from ${_basename(path)}',
            ),
          ),
        );
      },
      failure: (failure) {
        final message = switch (failure) {
          UnsupportedFailure() => 'Auto-fill not available for this format',
          _ => 'Could not read metadata: ${appFailureMessage(failure)}',
        };
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(message)),
        );
      },
    );
    setState(() {});
  }

  Future<String?> _defaultPickEbookFile() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['pdf', 'epub', 'mobi', 'azw3'],
    );
    return result?.files.single.path;
  }

  void _applyExtractedMeta(String path, ExtractedMeta meta) {
    if (_titleController.text.trim().isEmpty && meta.title != null) {
      _titleController.text = meta.title!;
    }
    if (_authorController.text.trim().isEmpty && meta.author != null) {
      _authorController.text = meta.author!;
    }
    if (_fileFormatController.text.trim().isEmpty) {
      _fileFormatController.text = meta.fileFormat ?? _extensionWithoutDot(path);
    }
  }

  String _basename(String path) => path.split(RegExp(r'[/\\]')).last;

  String _extensionWithoutDot(String path) {
    final dotIndex = path.lastIndexOf('.');
    if (dotIndex == -1 || dotIndex == path.length - 1) {
      return '';
    }
    return path.substring(dotIndex + 1).toLowerCase();
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

    switch (_resourceType) {
      case ResourceType.ebook:
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

      case ResourceType.webReader:
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

      case ResourceType.image:
        if (widget.resourceId != null) {
          context.read<ImageBloc>().add(
            UpdateImage(
              id,
              UpdateImageInput(
                title: _titleController.text,
                width: int.tryParse(_widthController.text),
                height: int.tryParse(_heightController.text),
                fileFormat: _fileFormatController.text.isEmpty
                    ? null
                    : _fileFormatController.text,
                fileSizeBytes: int.tryParse(_fileSizeBytesController.text),
              ),
            ),
          );
        } else {
          context.read<ImageBloc>().add(
            AddImage(
              NewImageInput(
                resource: resource,
                meta: ImageMeta(
                  resourceId: id,
                  width: int.tryParse(_widthController.text),
                  height: int.tryParse(_heightController.text),
                  fileFormat: _fileFormatController.text.isEmpty
                      ? null
                      : _fileFormatController.text,
                  fileSizeBytes: int.tryParse(_fileSizeBytesController.text),
                ),
              ),
            ),
          );
        }

      case ResourceType.video:
        if (widget.resourceId != null) {
          context.read<VideoBloc>().add(
            UpdateVideo(
              id,
              UpdateVideoInput(
                title: _titleController.text,
                durationSecs: int.tryParse(_durationController.text),
                fileFormat: _fileFormatController.text.isEmpty
                    ? null
                    : _fileFormatController.text,
                resolution: _resolutionController.text.isEmpty
                    ? null
                    : _resolutionController.text,
                fileSizeBytes: int.tryParse(_fileSizeBytesController.text),
              ),
            ),
          );
        } else {
          context.read<VideoBloc>().add(
            AddVideo(
              NewVideoInput(
                resource: resource,
                meta: VideoMeta(
                  resourceId: id,
                  durationSecs: int.tryParse(_durationController.text),
                  fileFormat: _fileFormatController.text.isEmpty
                      ? null
                      : _fileFormatController.text,
                  resolution: _resolutionController.text.isEmpty
                      ? null
                      : _resolutionController.text,
                  fileSizeBytes: int.tryParse(_fileSizeBytesController.text),
                ),
              ),
            ),
          );
        }

      case ResourceType.game:
        if (widget.resourceId != null) {
          context.read<GameBloc>().add(
            UpdateGame(
              id,
              UpdateGameInput(
                title: _titleController.text,
                platform: _platformController.text.isEmpty
                    ? null
                    : _platformController.text,
                store: _storeController.text.isEmpty
                    ? null
                    : _storeController.text,
                developer: _developerController.text.isEmpty
                    ? null
                    : _developerController.text,
                publisher: _publisherController.text.isEmpty
                    ? null
                    : _publisherController.text,
                manualNotes: _manualNotesController.text.isEmpty
                    ? null
                    : _manualNotesController.text,
              ),
            ),
          );
        } else {
          context.read<GameBloc>().add(
            AddGame(
              NewGameInput(
                resource: resource,
                meta: GameMeta(
                  resourceId: id,
                  platform: _platformController.text.isEmpty
                      ? null
                      : _platformController.text,
                  store: _storeController.text.isEmpty
                      ? null
                      : _storeController.text,
                  developer: _developerController.text.isEmpty
                      ? null
                      : _developerController.text,
                  publisher: _publisherController.text.isEmpty
                      ? null
                      : _publisherController.text,
                  manualNotes: _manualNotesController.text.isEmpty
                      ? null
                      : _manualNotesController.text,
                ),
              ),
            ),
          );
        }
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

  void _onImageOperationSuccess(ImageOperationSuccess state) {
    if (state.operationType == ImageOperationType.added ||
        state.operationType == ImageOperationType.updated) {
      final pendingResourceId = _pendingResourceId;
      final pendingLocation = _pendingLocation;
      if (pendingResourceId != null && pendingLocation != null) {
        context.read<ImageBloc>().add(AddImageLocation(pendingResourceId, pendingLocation));
        return;
      }
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(widget.resourceId == null ? 'Image added' : 'Image updated')),
      );
      return;
    }

    if (state.operationType == ImageOperationType.locationAdded) {
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(widget.resourceId == null ? 'Image added' : 'Image updated')),
      );
    }
  }

  void _onVideoOperationSuccess(VideoOperationSuccess state) {
    if (state.operationType == VideoOperationType.added ||
        state.operationType == VideoOperationType.updated) {
      final pendingResourceId = _pendingResourceId;
      final pendingLocation = _pendingLocation;
      if (pendingResourceId != null && pendingLocation != null) {
        context.read<VideoBloc>().add(AddVideoLocation(pendingResourceId, pendingLocation));
        return;
      }
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(widget.resourceId == null ? 'Video added' : 'Video updated')),
      );
      return;
    }

    if (state.operationType == VideoOperationType.locationAdded) {
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(widget.resourceId == null ? 'Video added' : 'Video updated')),
      );
    }
  }

  void _onGameOperationSuccess(GameOperationSuccess state) {
    if (state.operationType == GameOperationType.added ||
        state.operationType == GameOperationType.updated) {
      final pendingResourceId = _pendingResourceId;
      final pendingLocation = _pendingLocation;
      if (pendingResourceId != null && pendingLocation != null) {
        context.read<GameBloc>().add(AddGameLocation(pendingResourceId, pendingLocation));
        return;
      }
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(widget.resourceId == null ? 'Game added' : 'Game updated')),
      );
      return;
    }

    if (state.operationType == GameOperationType.locationAdded) {
      _pendingResourceId = null;
      _pendingLocation = null;
      Navigator.of(context).pop();
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(widget.resourceId == null ? 'Game added' : 'Game updated')),
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

  void _prefillFromImageDetail(ImageDetail detail) {
    if (_prefilled) {
      return;
    }
    _titleController.text = detail.resource.title;
    _notesController.text = detail.resource.notes ?? '';
    _widthController.text = detail.meta.width?.toString() ?? '';
    _heightController.text = detail.meta.height?.toString() ?? '';
    _fileFormatController.text = detail.meta.fileFormat ?? '';
    _fileSizeBytesController.text = detail.meta.fileSizeBytes?.toString() ?? '';
    if (detail.locations.isNotEmpty) {
      _deviceIdController.text = detail.locations.first.deviceId;
      _pathController.text = detail.locations.first.pathOrUrl;
      _storageType = detail.locations.first.storageType;
    }
    _prefilled = true;
    setState(() {});
  }

  void _prefillFromVideoDetail(VideoDetail detail) {
    if (_prefilled) {
      return;
    }
    _titleController.text = detail.resource.title;
    _notesController.text = detail.resource.notes ?? '';
    _durationController.text = detail.meta.durationSecs?.toString() ?? '';
    _fileFormatController.text = detail.meta.fileFormat ?? '';
    _resolutionController.text = detail.meta.resolution ?? '';
    _fileSizeBytesController.text = detail.meta.fileSizeBytes?.toString() ?? '';
    if (detail.locations.isNotEmpty) {
      _deviceIdController.text = detail.locations.first.deviceId;
      _pathController.text = detail.locations.first.pathOrUrl;
      _storageType = detail.locations.first.storageType;
    }
    _prefilled = true;
    setState(() {});
  }

  void _prefillFromGameDetail(GameDetail detail) {
    if (_prefilled) {
      return;
    }
    _titleController.text = detail.resource.title;
    _notesController.text = detail.resource.notes ?? '';
    _platformController.text = detail.meta.platform ?? '';
    _storeController.text = detail.meta.store ?? '';
    _developerController.text = detail.meta.developer ?? '';
    _publisherController.text = detail.meta.publisher ?? '';
    _manualNotesController.text = detail.meta.manualNotes ?? '';
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
        BlocListener<ImageBloc, ImageState>(
          listener: (context, state) {
            if (state is ImageDetailLoaded && widget.resourceId == state.image.resource.id) {
              _prefillFromImageDetail(state.image);
            }
            if (state is ImageOperationSuccess) {
              _onImageOperationSuccess(state);
            }
            if (state is ImageError) {
              _pendingResourceId = null;
              _pendingLocation = null;
              _showFailure(state.failure);
            }
          },
        ),
        BlocListener<VideoBloc, VideoState>(
          listener: (context, state) {
            if (state is VideoDetailLoaded && widget.resourceId == state.video.resource.id) {
              _prefillFromVideoDetail(state.video);
            }
            if (state is VideoOperationSuccess) {
              _onVideoOperationSuccess(state);
            }
            if (state is VideoError) {
              _pendingResourceId = null;
              _pendingLocation = null;
              _showFailure(state.failure);
            }
          },
        ),
        BlocListener<GameBloc, GameState>(
          listener: (context, state) {
            if (state is GameDetailLoaded && widget.resourceId == state.game.resource.id) {
              _prefillFromGameDetail(state.game);
            }
            if (state is GameOperationSuccess) {
              _onGameOperationSuccess(state);
            }
            if (state is GameError) {
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
                DropdownMenuItem(
                  value: ResourceType.image,
                  child: Text('Image'),
                ),
                DropdownMenuItem(
                  value: ResourceType.video,
                  child: Text('Video'),
                ),
                DropdownMenuItem(
                  value: ResourceType.game,
                  child: Text('Game'),
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
              const SizedBox(height: 12),
              Align(
                alignment: Alignment.centerLeft,
                child: FilledButton.icon(
                  key: const Key('pick-ebook-file'),
                  onPressed: _pickAndExtractMetadata,
                  icon: const Icon(Icons.file_open),
                  label: const Text('Pick file'),
                ),
              ),
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
            ] else if (_resourceType == ResourceType.webReader) ...[
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
            ] else if (_resourceType == ResourceType.image) ...[
              TextField(
                key: const Key('image-width'),
                controller: _widthController,
                decoration: const InputDecoration(labelText: 'Width'),
                keyboardType: TextInputType.number,
              ),
              TextField(
                key: const Key('image-height'),
                controller: _heightController,
                decoration: const InputDecoration(labelText: 'Height'),
                keyboardType: TextInputType.number,
              ),
              TextField(
                key: const Key('image-file-format'),
                controller: _fileFormatController,
                decoration: const InputDecoration(labelText: 'File format'),
              ),
              TextField(
                key: const Key('image-file-size'),
                controller: _fileSizeBytesController,
                decoration: const InputDecoration(labelText: 'File size (bytes)'),
                keyboardType: TextInputType.number,
              ),
            ] else if (_resourceType == ResourceType.video) ...[
              TextField(
                key: const Key('video-duration'),
                controller: _durationController,
                decoration: const InputDecoration(labelText: 'Duration (secs)'),
                keyboardType: TextInputType.number,
              ),
              TextField(
                key: const Key('video-file-format'),
                controller: _fileFormatController,
                decoration: const InputDecoration(labelText: 'File format'),
              ),
              TextField(
                key: const Key('video-resolution'),
                controller: _resolutionController,
                decoration: const InputDecoration(labelText: 'Resolution'),
              ),
              TextField(
                key: const Key('video-file-size'),
                controller: _fileSizeBytesController,
                decoration: const InputDecoration(labelText: 'File size (bytes)'),
                keyboardType: TextInputType.number,
              ),
            ] else if (_resourceType == ResourceType.game) ...[
              TextField(
                key: const Key('game-platform'),
                controller: _platformController,
                decoration: const InputDecoration(labelText: 'Platform'),
              ),
              TextField(
                key: const Key('game-store'),
                controller: _storeController,
                decoration: const InputDecoration(labelText: 'Store'),
              ),
              TextField(
                key: const Key('game-developer'),
                controller: _developerController,
                decoration: const InputDecoration(labelText: 'Developer'),
              ),
              TextField(
                key: const Key('game-publisher'),
                controller: _publisherController,
                decoration: const InputDecoration(labelText: 'Publisher'),
              ),
              TextField(
                key: const Key('game-manual-notes'),
                controller: _manualNotesController,
                decoration: const InputDecoration(labelText: 'Manual notes'),
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
