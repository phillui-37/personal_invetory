import '../models/batch_operations.dart';
import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../models/result.dart';
import 'batch_operation_repository.dart';
import 'ebook_repository.dart';
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
}

class InMemoryWebReaderRepository implements WebReaderRepository {
  final Map<String, WebReaderDetail> _items = <String, WebReaderDetail>{};

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
