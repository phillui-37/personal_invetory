import '../models/batch_operations.dart';
import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../models/result.dart';
import 'batch_operation_repository.dart';
import 'ebook_repository.dart';
import 'game_repository.dart';
import 'image_repository.dart';
import 'video_repository.dart';
import 'web_reader_repository.dart';

class InMemoryEbookRepository implements EbookRepository {
  final Map<String, EbookDetail> _items = <String, EbookDetail>{};

  @override
  Future<Result<Resource, AppFailure>> addEbook(NewEbookInput input) async {
    final detail = EbookDetail(resource: input.resource, meta: input.meta, locations: const []);
    _items[input.resource.id] = detail;
    return Success<Resource, AppFailure>(input.resource);
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return const Failure<ResourceLocation, AppFailure>(NotFoundFailure('resource'));
    }
    final location = ResourceLocation(
      id: '${resourceId}_${detail.locations.length + 1}',
      resourceId: resourceId,
      deviceId: input.deviceId,
      pathOrUrl: input.pathOrUrl,
      storageType: input.storageType,
    );
    _items[resourceId] = EbookDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: [...detail.locations, location],
    );
    return Success<ResourceLocation, AppFailure>(location);
  }

  @override
  Future<Result<void, AppFailure>> deleteEbook(String id) async {
    _items.remove(id);
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<EbookDetail, AppFailure>> getEbook(String id) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<EbookDetail, AppFailure>(NotFoundFailure(id));
    }
    return Success<EbookDetail, AppFailure>(detail);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listEbooks() async {
    return Success<List<Resource>, AppFailure>(
      _items.values.map((detail) => detail.resource).toList(),
    );
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return Failure<void, AppFailure>(NotFoundFailure(resourceId));
    }
    _items[resourceId] = EbookDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: detail.locations.where((location) => location.id != locationId).toList(),
    );
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchEbooks(String query) async {
    final lower = query.toLowerCase();
    return Success<List<Resource>, AppFailure>(
      _items.values
          .map((detail) => detail.resource)
          .where((resource) => resource.title.toLowerCase().contains(lower))
          .toList(),
    );
  }

  @override
  Future<Result<Resource, AppFailure>> updateEbook(String id, UpdateEbookInput input) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<Resource, AppFailure>(NotFoundFailure(id));
    }
    final updated = Resource(
      id: detail.resource.id,
      title: input.title ?? detail.resource.title,
      notes: detail.resource.notes,
      resourceType: detail.resource.resourceType,
    );
    _items[id] = EbookDetail(
      resource: updated,
      meta: EbookMeta(
        resourceId: detail.meta.resourceId,
        author: input.author ?? detail.meta.author,
        fileFormat: input.fileFormat ?? detail.meta.fileFormat,
        totalPages: input.totalPages ?? detail.meta.totalPages,
        currentPage: input.currentPage ?? detail.meta.currentPage,
      ),
      locations: detail.locations,
    );
    return Success<Resource, AppFailure>(updated);
  }

  @override
  Future<Result<BatchImportResult, AppFailure>> batchImport(
    List<BatchImportEntry> entries,
  ) async {
    final succeeded = <String>[];
    final failed = <BatchImportFailureItem>[];

    for (var i = 0; i < entries.length; i++) {
      final entry = entries[i];
      if (entry.title.trim().isEmpty) {
        failed.add(
          BatchImportFailureItem(index: i, error: 'title must not be empty'),
        );
        continue;
      }

      final id = 'resource-${DateTime.now().microsecondsSinceEpoch}-$i';
      final resource = Resource(
        id: id,
        title: entry.title,
        resourceType: ResourceType.ebook,
      );
      _items[id] = EbookDetail(
        resource: resource,
        meta: EbookMeta(
          resourceId: id,
          author: entry.author,
          fileFormat: entry.fileFormat,
        ),
        locations: entry.filePath == null
            ? const []
            : [
                ResourceLocation(
                  id: '$id-loc-1',
                  resourceId: id,
                  deviceId: 'imported',
                  pathOrUrl: entry.filePath!,
                  storageType: StorageType.localFs,
                ),
              ],
      );
      succeeded.add(id);
    }

    return Success(BatchImportResult(succeeded: succeeded, failed: failed));
  }
}

class InMemoryWebReaderRepository implements WebReaderRepository {
  final Map<String, WebReaderDetail> _items = <String, WebReaderDetail>{};
  final Map<String, List<ChapterCheck>> _checkHistory =
      <String, List<ChapterCheck>>{};

  @override
  Future<Result<Resource, AppFailure>> addWebReader(NewWebReaderInput input) async {
    final detail = WebReaderDetail(
      resource: input.resource,
      meta: input.meta,
      locations: const [],
    );
    _items[input.resource.id] = detail;
    return Success<Resource, AppFailure>(input.resource);
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return const Failure<ResourceLocation, AppFailure>(NotFoundFailure('resource'));
    }
    final location = ResourceLocation(
      id: '${resourceId}_${detail.locations.length + 1}',
      resourceId: resourceId,
      deviceId: input.deviceId,
      pathOrUrl: input.pathOrUrl,
      storageType: input.storageType,
    );
    _items[resourceId] = WebReaderDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: [...detail.locations, location],
    );
    return Success<ResourceLocation, AppFailure>(location);
  }

  @override
  Future<Result<void, AppFailure>> deleteWebReader(String id) async {
    _items.remove(id);
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<WebReaderDetail, AppFailure>> getWebReader(String id) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<WebReaderDetail, AppFailure>(NotFoundFailure(id));
    }
    return Success<WebReaderDetail, AppFailure>(detail);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listWebReaders() async {
    return Success<List<Resource>, AppFailure>(
      _items.values.map((detail) => detail.resource).toList(),
    );
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return Failure<void, AppFailure>(NotFoundFailure(resourceId));
    }
    _items[resourceId] = WebReaderDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: detail.locations.where((location) => location.id != locationId).toList(),
    );
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchWebReaders(String query) async {
    final lower = query.toLowerCase();
    return Success<List<Resource>, AppFailure>(
      _items.values
          .map((detail) => detail.resource)
          .where((resource) => resource.title.toLowerCase().contains(lower))
          .toList(),
    );
  }

  @override
  Future<Result<void, AppFailure>> trackProgress(WebReaderProgressSignal signal) async {
    final detail = _items[signal.resourceId];
    if (detail == null) {
      return Failure<void, AppFailure>(NotFoundFailure(signal.resourceId));
    }
    _items[signal.resourceId] = WebReaderDetail(
      resource: detail.resource,
      meta: WebReaderMeta(
        resourceId: detail.meta.resourceId,
        url: signal.url,
        siteName: detail.meta.siteName,
        lastReadChapter: signal.domChapter ?? detail.meta.lastReadChapter,
        progress: signal.domProgress ?? detail.meta.progress,
      ),
      locations: detail.locations,
    );
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<ChapterCheck, AppFailure>> triggerCheck(
    String resourceId,
  ) async {
    if (!_items.containsKey(resourceId)) {
      return Failure<ChapterCheck, AppFailure>(NotFoundFailure(resourceId));
    }
    final check = ChapterCheck(
      id: '${resourceId}_check_${DateTime.now().millisecondsSinceEpoch}',
      resourceId: resourceId,
      hasNewChapter: false,
      checkedAt: DateTime.now(),
    );
    _checkHistory.putIfAbsent(resourceId, () => []).add(check);
    return Success<ChapterCheck, AppFailure>(check);
  }

  @override
  Future<Result<List<ChapterCheck>, AppFailure>> listCheckHistory(
    String resourceId,
  ) async {
    return Success<List<ChapterCheck>, AppFailure>(
      List.unmodifiable(_checkHistory[resourceId] ?? []),
    );
  }

  @override
  Future<Result<Resource, AppFailure>> updateWebReader(
    String id,
    UpdateWebReaderInput input,
  ) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<Resource, AppFailure>(NotFoundFailure(id));
    }
    final updated = Resource(
      id: detail.resource.id,
      title: input.title ?? detail.resource.title,
      notes: detail.resource.notes,
      resourceType: detail.resource.resourceType,
    );
    _items[id] = WebReaderDetail(
      resource: updated,
      meta: WebReaderMeta(
        resourceId: detail.meta.resourceId,
        url: input.url ?? detail.meta.url,
        siteName: input.siteName ?? detail.meta.siteName,
        lastReadChapter: input.lastReadChapter ?? detail.meta.lastReadChapter,
        progress: input.progress ?? detail.meta.progress,
      ),
      locations: detail.locations,
    );
    return Success<Resource, AppFailure>(updated);
  }
}

class InMemoryBatchOperationRepository implements BatchOperationRepository {
  const InMemoryBatchOperationRepository();

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchCopyMetadata(
    BatchMetadataCopyRequest request,
  ) async {
    return Success<BatchOperationResponse, AppFailure>(
      BatchOperationResponse(
        type: BatchOperationType.copyMetadata,
        results: request.targetResourceIds
            .map((targetId) => BatchOperationItemResult(itemKey: targetId, success: true))
            .toList(),
      ),
    );
  }

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchImport(
    BatchImportRequest request,
  ) async {
    return Success<BatchOperationResponse, AppFailure>(
      BatchOperationResponse(
        type: BatchOperationType.importResources,
        results: request.paths
            .map((path) => BatchOperationItemResult(itemKey: path, success: true))
            .toList(),
      ),
    );
  }

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchUpdateMetadata(
    BatchMetadataUpdateRequest request,
  ) async {
    return Success<BatchOperationResponse, AppFailure>(
      BatchOperationResponse(
        type: BatchOperationType.updateMetadata,
        results: request.resourceIds
            .map((resourceId) => BatchOperationItemResult(itemKey: resourceId, success: true))
            .toList(),
      ),
    );
  }
}

class InMemoryImageRepository implements ImageRepository {
  final Map<String, ImageDetail> _items = <String, ImageDetail>{};

  @override
  Future<Result<Resource, AppFailure>> addImage(NewImageInput input) async {
    final detail = ImageDetail(resource: input.resource, meta: input.meta, locations: const []);
    _items[input.resource.id] = detail;
    return Success<Resource, AppFailure>(input.resource);
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return const Failure<ResourceLocation, AppFailure>(NotFoundFailure('resource'));
    }
    final location = ResourceLocation(
      id: '${resourceId}_${detail.locations.length + 1}',
      resourceId: resourceId,
      deviceId: input.deviceId,
      pathOrUrl: input.pathOrUrl,
      storageType: input.storageType,
    );
    _items[resourceId] = ImageDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: [...detail.locations, location],
    );
    return Success<ResourceLocation, AppFailure>(location);
  }

  @override
  Future<Result<void, AppFailure>> deleteImage(String id) async {
    _items.remove(id);
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<ImageDetail, AppFailure>> getImage(String id) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<ImageDetail, AppFailure>(NotFoundFailure(id));
    }
    return Success<ImageDetail, AppFailure>(detail);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listImages() async {
    return Success<List<Resource>, AppFailure>(
      _items.values.map((detail) => detail.resource).toList(),
    );
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return Failure<void, AppFailure>(NotFoundFailure(resourceId));
    }
    _items[resourceId] = ImageDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: detail.locations.where((location) => location.id != locationId).toList(),
    );
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchImages(String query) async {
    final lower = query.toLowerCase();
    return Success<List<Resource>, AppFailure>(
      _items.values
          .map((detail) => detail.resource)
          .where((resource) => resource.title.toLowerCase().contains(lower))
          .toList(),
    );
  }

  @override
  Future<Result<Resource, AppFailure>> updateImage(String id, UpdateImageInput input) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<Resource, AppFailure>(NotFoundFailure(id));
    }
    final updated = Resource(
      id: detail.resource.id,
      title: input.title ?? detail.resource.title,
      notes: detail.resource.notes,
      resourceType: detail.resource.resourceType,
    );
    _items[id] = ImageDetail(
      resource: updated,
      meta: ImageMeta(
        resourceId: detail.meta.resourceId,
        width: input.width ?? detail.meta.width,
        height: input.height ?? detail.meta.height,
        fileFormat: input.fileFormat ?? detail.meta.fileFormat,
        fileSizeBytes: input.fileSizeBytes ?? detail.meta.fileSizeBytes,
      ),
      locations: detail.locations,
    );
    return Success<Resource, AppFailure>(updated);
  }
}

class InMemoryVideoRepository implements VideoRepository {
  final Map<String, VideoDetail> _items = <String, VideoDetail>{};

  @override
  Future<Result<Resource, AppFailure>> addVideo(NewVideoInput input) async {
    final detail = VideoDetail(resource: input.resource, meta: input.meta, locations: const []);
    _items[input.resource.id] = detail;
    return Success<Resource, AppFailure>(input.resource);
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return const Failure<ResourceLocation, AppFailure>(NotFoundFailure('resource'));
    }
    final location = ResourceLocation(
      id: '${resourceId}_${detail.locations.length + 1}',
      resourceId: resourceId,
      deviceId: input.deviceId,
      pathOrUrl: input.pathOrUrl,
      storageType: input.storageType,
    );
    _items[resourceId] = VideoDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: [...detail.locations, location],
    );
    return Success<ResourceLocation, AppFailure>(location);
  }

  @override
  Future<Result<void, AppFailure>> deleteVideo(String id) async {
    _items.remove(id);
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<VideoDetail, AppFailure>> getVideo(String id) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<VideoDetail, AppFailure>(NotFoundFailure(id));
    }
    return Success<VideoDetail, AppFailure>(detail);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listVideos() async {
    return Success<List<Resource>, AppFailure>(
      _items.values.map((detail) => detail.resource).toList(),
    );
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return Failure<void, AppFailure>(NotFoundFailure(resourceId));
    }
    _items[resourceId] = VideoDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: detail.locations.where((location) => location.id != locationId).toList(),
    );
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchVideos(String query) async {
    final lower = query.toLowerCase();
    return Success<List<Resource>, AppFailure>(
      _items.values
          .map((detail) => detail.resource)
          .where((resource) => resource.title.toLowerCase().contains(lower))
          .toList(),
    );
  }

  @override
  Future<Result<Resource, AppFailure>> updateVideo(String id, UpdateVideoInput input) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<Resource, AppFailure>(NotFoundFailure(id));
    }
    final updated = Resource(
      id: detail.resource.id,
      title: input.title ?? detail.resource.title,
      notes: detail.resource.notes,
      resourceType: detail.resource.resourceType,
    );
    _items[id] = VideoDetail(
      resource: updated,
      meta: VideoMeta(
        resourceId: detail.meta.resourceId,
        durationSecs: input.durationSecs ?? detail.meta.durationSecs,
        fileFormat: input.fileFormat ?? detail.meta.fileFormat,
        resolution: input.resolution ?? detail.meta.resolution,
        fileSizeBytes: input.fileSizeBytes ?? detail.meta.fileSizeBytes,
      ),
      locations: detail.locations,
    );
    return Success<Resource, AppFailure>(updated);
  }
}

class InMemoryGameRepository implements GameRepository {
  final Map<String, GameDetail> _items = <String, GameDetail>{};

  @override
  Future<Result<Resource, AppFailure>> addGame(NewGameInput input) async {
    final detail = GameDetail(resource: input.resource, meta: input.meta, locations: const []);
    _items[input.resource.id] = detail;
    return Success<Resource, AppFailure>(input.resource);
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return const Failure<ResourceLocation, AppFailure>(NotFoundFailure('resource'));
    }
    final location = ResourceLocation(
      id: '${resourceId}_${detail.locations.length + 1}',
      resourceId: resourceId,
      deviceId: input.deviceId,
      pathOrUrl: input.pathOrUrl,
      storageType: input.storageType,
    );
    _items[resourceId] = GameDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: [...detail.locations, location],
    );
    return Success<ResourceLocation, AppFailure>(location);
  }

  @override
  Future<Result<void, AppFailure>> deleteGame(String id) async {
    _items.remove(id);
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<GameDetail, AppFailure>> getGame(String id) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<GameDetail, AppFailure>(NotFoundFailure(id));
    }
    return Success<GameDetail, AppFailure>(detail);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listGames() async {
    return Success<List<Resource>, AppFailure>(
      _items.values.map((detail) => detail.resource).toList(),
    );
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) async {
    final detail = _items[resourceId];
    if (detail == null) {
      return Failure<void, AppFailure>(NotFoundFailure(resourceId));
    }
    _items[resourceId] = GameDetail(
      resource: detail.resource,
      meta: detail.meta,
      locations: detail.locations.where((location) => location.id != locationId).toList(),
    );
    return const Success<void, AppFailure>(null);
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchGames(String query) async {
    final lower = query.toLowerCase();
    return Success<List<Resource>, AppFailure>(
      _items.values
          .map((detail) => detail.resource)
          .where((resource) => resource.title.toLowerCase().contains(lower))
          .toList(),
    );
  }

  @override
  Future<Result<Resource, AppFailure>> updateGame(String id, UpdateGameInput input) async {
    final detail = _items[id];
    if (detail == null) {
      return Failure<Resource, AppFailure>(NotFoundFailure(id));
    }
    final updated = Resource(
      id: detail.resource.id,
      title: input.title ?? detail.resource.title,
      notes: detail.resource.notes,
      resourceType: detail.resource.resourceType,
    );
    _items[id] = GameDetail(
      resource: updated,
      meta: GameMeta(
        resourceId: detail.meta.resourceId,
        platform: input.platform ?? detail.meta.platform,
        store: input.store ?? detail.meta.store,
        developer: input.developer ?? detail.meta.developer,
        publisher: input.publisher ?? detail.meta.publisher,
        manualNotes: input.manualNotes ?? detail.meta.manualNotes,
      ),
      locations: detail.locations,
    );
    return Success<Resource, AppFailure>(updated);
  }
}
