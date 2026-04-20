import 'package:personal_inventory_frontend/models/dedup.dart';
import 'package:personal_inventory_frontend/models/device.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/progress.dart';
import 'package:personal_inventory_frontend/models/batch_operations.dart';
import 'package:personal_inventory_frontend/models/repository_inputs.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/tag.dart';
import 'package:personal_inventory_frontend/models/vault.dart';
import 'package:personal_inventory_frontend/repositories/dedup_repository.dart';
import 'package:personal_inventory_frontend/repositories/device_repository.dart';
import 'package:personal_inventory_frontend/repositories/batch_operation_repository.dart';
import 'package:personal_inventory_frontend/repositories/ebook_repository.dart';
import 'package:personal_inventory_frontend/repositories/game_repository.dart';
import 'package:personal_inventory_frontend/repositories/image_repository.dart';
import 'package:personal_inventory_frontend/repositories/progress_repository.dart';
import 'package:personal_inventory_frontend/repositories/tag_repository.dart';
import 'package:personal_inventory_frontend/repositories/vault_repository.dart';
import 'package:personal_inventory_frontend/repositories/video_repository.dart';
import 'package:personal_inventory_frontend/repositories/web_reader_repository.dart';

class FakeProgressRepository implements ProgressRepository {
  FakeProgressRepository({
    this.getResult = const Success<ResourceProgress?, AppFailure>(null),
    this.upsertResult,
  });

  Result<ResourceProgress?, AppFailure> getResult;
  Result<ResourceProgress, AppFailure>? upsertResult;

  int getCalls = 0;
  int upsertCalls = 0;
  ResourceType? lastGetResourceType;
  String? lastGetResourceId;
  ResourceType? lastUpsertResourceType;
  String? lastUpsertResourceId;
  double? lastUpsertProgress;
  String? lastUpsertNotes;

  @override
  Future<Result<ResourceProgress?, AppFailure>> getProgress(
    ResourceType resourceType,
    String resourceId,
  ) async {
    getCalls += 1;
    lastGetResourceType = resourceType;
    lastGetResourceId = resourceId;
    return getResult;
  }

  @override
  Future<Result<ResourceProgress, AppFailure>> upsertProgress(
    ResourceType resourceType,
    String resourceId,
    double progress, {
    String? notes,
  }) async {
    upsertCalls += 1;
    lastUpsertResourceType = resourceType;
    lastUpsertResourceId = resourceId;
    lastUpsertProgress = progress;
    lastUpsertNotes = notes;
    return upsertResult ??
        Success<ResourceProgress, AppFailure>(
          ResourceProgress(
            resourceId: resourceId,
            progress: progress,
            notes: notes,
            updatedAt: DateTime.now().toUtc(),
          ),
        );
  }
}

class FakeEbookRepository implements EbookRepository {
  FakeEbookRepository({
    this.listResult = const Success([]),
    this.searchResult = const Success([]),
    this.detailResult,
    this.addResult,
    this.updateResult,
    this.deleteResult = const Success(null),
    this.addLocationResult,
    this.removeLocationResult = const Success(null),
    this.batchImportResult = const Success(
      BatchImportResult(succeeded: [], failed: []),
    ),
  });

  Result<List<Resource>, AppFailure> listResult;
  Result<List<Resource>, AppFailure> searchResult;
  Result<EbookDetail, AppFailure>? detailResult;
  Result<Resource, AppFailure>? addResult;
  Result<Resource, AppFailure>? updateResult;
  Result<void, AppFailure> deleteResult;
  Result<ResourceLocation, AppFailure>? addLocationResult;
  Result<void, AppFailure> removeLocationResult;
  Result<BatchImportResult, AppFailure> batchImportResult;

  int listCalls = 0;
  int searchCalls = 0;
  int deleteCalls = 0;
  int addCalls = 0;
  int updateCalls = 0;
  int addLocationCalls = 0;
  int batchImportCalls = 0;
  String? lastUpdateId;
  UpdateEbookInput? lastUpdateInput;
  String? lastAddLocationResourceId;
  NewLocationInput? lastAddLocationInput;
  NewEbookInput? lastAddInput;
  List<BatchImportEntry>? lastBatchImportEntries;

  @override
  Future<Result<Resource, AppFailure>> addEbook(NewEbookInput input) async {
    addCalls += 1;
    lastAddInput = input;
    return addResult ?? const Failure(ServerFailure(500));
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    addLocationCalls += 1;
    lastAddLocationResourceId = resourceId;
    lastAddLocationInput = input;
    return addLocationResult ??
        Success(
          ResourceLocation(
            id: 'loc-new',
            resourceId: resourceId,
            deviceId: input.deviceId,
            pathOrUrl: input.pathOrUrl,
            storageType: input.storageType,
          ),
        );
  }

  @override
  Future<Result<void, AppFailure>> deleteEbook(String id) async {
    deleteCalls += 1;
    return deleteResult;
  }

  @override
  Future<Result<EbookDetail, AppFailure>> getEbook(String id) async {
    return detailResult ?? const Failure(NotFoundFailure('missing'));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listEbooks() async {
    listCalls += 1;
    return listResult;
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(
    String resourceId,
    String locationId,
  ) async {
    return removeLocationResult;
  }

  @override
  Future<Result<BatchImportResult, AppFailure>> batchImport(
    List<BatchImportEntry> entries,
  ) async {
    batchImportCalls += 1;
    lastBatchImportEntries = entries;
    return batchImportResult;
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchEbooks(String query) async {
    searchCalls += 1;
    return searchResult;
  }

  @override
  Future<Result<Resource, AppFailure>> updateEbook(
    String id,
    UpdateEbookInput input,
  ) async {
    updateCalls += 1;
    lastUpdateId = id;
    lastUpdateInput = input;
    return updateResult ?? const Failure(ServerFailure(500));
  }
}

class FakeTagRepository implements TagRepository {
  FakeTagRepository({
    this.listResult = const Success<List<Tag>, AppFailure>([]),
    this.createResult,
    this.deleteResult = const Success<void, AppFailure>(null),
    this.tagsForResourceResult = const Success<List<Tag>, AppFailure>([]),
    this.attachResult = const Success<void, AppFailure>(null),
    this.detachResult = const Success<void, AppFailure>(null),
  });

  Result<List<Tag>, AppFailure> listResult;
  Result<Tag, AppFailure>? createResult;
  Result<void, AppFailure> deleteResult;
  Result<List<Tag>, AppFailure> tagsForResourceResult;
  Result<void, AppFailure> attachResult;
  Result<void, AppFailure> detachResult;

  int listCalls = 0;
  int createCalls = 0;
  int deleteCalls = 0;
  int tagsForResourceCalls = 0;
  int attachCalls = 0;
  int detachCalls = 0;
  String? lastCreateName;
  String? lastDeleteId;
  ResourceType? lastTagsForResourceType;
  String? lastTagsForResourceId;
  ResourceType? lastAttachType;
  String? lastAttachResourceId;
  String? lastAttachTagId;
  ResourceType? lastDetachType;
  String? lastDetachResourceId;
  String? lastDetachTagId;

  @override
  Future<Result<List<Tag>, AppFailure>> listTags() async {
    listCalls += 1;
    return listResult;
  }

  @override
  Future<Result<Tag, AppFailure>> createTag(String name) async {
    createCalls += 1;
    lastCreateName = name;
    return createResult ??
        Success<Tag, AppFailure>(
          Tag(
            id: 'tag-1',
            name: name.trim().toLowerCase(),
            createdAt: DateTime.now().toUtc(),
          ),
        );
  }

  @override
  Future<Result<void, AppFailure>> deleteTag(String id) async {
    deleteCalls += 1;
    lastDeleteId = id;
    return deleteResult;
  }

  @override
  Future<Result<List<Tag>, AppFailure>> tagsForResource(
    ResourceType resourceType,
    String resourceId,
  ) async {
    tagsForResourceCalls += 1;
    lastTagsForResourceType = resourceType;
    lastTagsForResourceId = resourceId;
    return tagsForResourceResult;
  }

  @override
  Future<Result<void, AppFailure>> attachTag(
    ResourceType resourceType,
    String resourceId,
    String tagId,
  ) async {
    attachCalls += 1;
    lastAttachType = resourceType;
    lastAttachResourceId = resourceId;
    lastAttachTagId = tagId;
    return attachResult;
  }

  @override
  Future<Result<void, AppFailure>> detachTag(
    ResourceType resourceType,
    String resourceId,
    String tagId,
  ) async {
    detachCalls += 1;
    lastDetachType = resourceType;
    lastDetachResourceId = resourceId;
    lastDetachTagId = tagId;
    return detachResult;
  }
}

class FakeWebReaderRepository implements WebReaderRepository {
  FakeWebReaderRepository({
    this.listResult = const Success([]),
    this.searchResult = const Success([]),
    this.detailResult,
    this.addResult,
    this.updateResult,
    this.deleteResult = const Success(null),
    this.addLocationResult,
    this.removeLocationResult = const Success(null),
    this.trackProgressResult = const Success(null),
    this.triggerCheckResult,
    this.listCheckHistoryResult = const Success([]),
  });

  Result<List<Resource>, AppFailure> listResult;
  Result<List<Resource>, AppFailure> searchResult;
  Result<WebReaderDetail, AppFailure>? detailResult;
  Result<Resource, AppFailure>? addResult;
  Result<Resource, AppFailure>? updateResult;
  Result<void, AppFailure> deleteResult;
  Result<ResourceLocation, AppFailure>? addLocationResult;
  Result<void, AppFailure> removeLocationResult;
  Result<void, AppFailure> trackProgressResult;
  Result<ChapterCheck, AppFailure>? triggerCheckResult;
  Result<List<ChapterCheck>, AppFailure> listCheckHistoryResult;

  int listCalls = 0;
  int searchCalls = 0;
  int trackProgressCalls = 0;
  int deleteCalls = 0;
  int addCalls = 0;
  int updateCalls = 0;
  int addLocationCalls = 0;
  int triggerCheckCalls = 0;
  int listCheckHistoryCalls = 0;
  String? lastUpdateId;
  UpdateWebReaderInput? lastUpdateInput;
  String? lastAddLocationResourceId;
  NewLocationInput? lastAddLocationInput;
  NewWebReaderInput? lastAddInput;
  String? lastTriggerCheckResourceId;
  String? lastListCheckHistoryResourceId;

  @override
  Future<Result<Resource, AppFailure>> addWebReader(NewWebReaderInput input) async {
    addCalls += 1;
    lastAddInput = input;
    return addResult ?? const Failure(ServerFailure(500));
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    addLocationCalls += 1;
    lastAddLocationResourceId = resourceId;
    lastAddLocationInput = input;
    return addLocationResult ??
        Success(
          ResourceLocation(
            id: 'loc-new',
            resourceId: resourceId,
            deviceId: input.deviceId,
            pathOrUrl: input.pathOrUrl,
            storageType: input.storageType,
          ),
        );
  }

  @override
  Future<Result<void, AppFailure>> deleteWebReader(String id) async {
    deleteCalls += 1;
    return deleteResult;
  }

  @override
  Future<Result<WebReaderDetail, AppFailure>> getWebReader(String id) async {
    return detailResult ?? const Failure(NotFoundFailure('missing'));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listWebReaders() async {
    listCalls += 1;
    return listResult;
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(
    String resourceId,
    String locationId,
  ) async {
    return removeLocationResult;
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchWebReaders(String query) async {
    searchCalls += 1;
    return searchResult;
  }

  @override
  Future<Result<void, AppFailure>> trackProgress(WebReaderProgressSignal signal) async {
    trackProgressCalls += 1;
    return trackProgressResult;
  }

  @override
  Future<Result<ChapterCheck, AppFailure>> triggerCheck(
    String resourceId,
  ) async {
    triggerCheckCalls += 1;
    lastTriggerCheckResourceId = resourceId;
    return triggerCheckResult ?? const Failure(ServerFailure(500));
  }

  @override
  Future<Result<List<ChapterCheck>, AppFailure>> listCheckHistory(
    String resourceId,
  ) async {
    listCheckHistoryCalls += 1;
    lastListCheckHistoryResourceId = resourceId;
    return listCheckHistoryResult;
  }

  @override
  Future<Result<Resource, AppFailure>> updateWebReader(
    String id,
    UpdateWebReaderInput input,
  ) async {
    updateCalls += 1;
    lastUpdateId = id;
    lastUpdateInput = input;
    return updateResult ?? const Failure(ServerFailure(500));
  }
}

class FakeImageRepository implements ImageRepository {
  FakeImageRepository({
    this.listResult = const Success([]),
    this.searchResult = const Success([]),
    this.detailResult,
    this.addResult,
    this.updateResult,
    this.deleteResult = const Success(null),
    this.addLocationResult,
    this.removeLocationResult = const Success(null),
  });

  Result<List<Resource>, AppFailure> listResult;
  Result<List<Resource>, AppFailure> searchResult;
  Result<ImageDetail, AppFailure>? detailResult;
  Result<Resource, AppFailure>? addResult;
  Result<Resource, AppFailure>? updateResult;
  Result<void, AppFailure> deleteResult;
  Result<ResourceLocation, AppFailure>? addLocationResult;
  Result<void, AppFailure> removeLocationResult;

  int listCalls = 0;
  int addCalls = 0;
  int updateCalls = 0;
  int addLocationCalls = 0;
  NewImageInput? lastAddInput;

  @override
  Future<Result<Resource, AppFailure>> addImage(NewImageInput input) async {
    addCalls += 1;
    lastAddInput = input;
    return addResult ?? const Failure(ServerFailure(500));
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    addLocationCalls += 1;
    return addLocationResult ??
        Success(
          ResourceLocation(
            id: 'loc-new',
            resourceId: resourceId,
            deviceId: input.deviceId,
            pathOrUrl: input.pathOrUrl,
            storageType: input.storageType,
          ),
        );
  }

  @override
  Future<Result<void, AppFailure>> deleteImage(String id) async => deleteResult;

  @override
  Future<Result<ImageDetail, AppFailure>> getImage(String id) async {
    return detailResult ?? const Failure(NotFoundFailure('missing'));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listImages() async {
    listCalls += 1;
    return listResult;
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(
    String resourceId,
    String locationId,
  ) async => removeLocationResult;

  @override
  Future<Result<List<Resource>, AppFailure>> searchImages(String query) async => searchResult;

  @override
  Future<Result<Resource, AppFailure>> updateImage(
    String id,
    UpdateImageInput input,
  ) async => updateResult ?? const Failure(ServerFailure(500));
}

class FakeVideoRepository implements VideoRepository {
  FakeVideoRepository({
    this.listResult = const Success([]),
    this.searchResult = const Success([]),
    this.detailResult,
    this.addResult,
    this.updateResult,
    this.deleteResult = const Success(null),
    this.addLocationResult,
    this.removeLocationResult = const Success(null),
  });

  Result<List<Resource>, AppFailure> listResult;
  Result<List<Resource>, AppFailure> searchResult;
  Result<VideoDetail, AppFailure>? detailResult;
  Result<Resource, AppFailure>? addResult;
  Result<Resource, AppFailure>? updateResult;
  Result<void, AppFailure> deleteResult;
  Result<ResourceLocation, AppFailure>? addLocationResult;
  Result<void, AppFailure> removeLocationResult;

  int listCalls = 0;
  int addCalls = 0;
  int updateCalls = 0;
  int addLocationCalls = 0;
  NewVideoInput? lastAddInput;

  @override
  Future<Result<Resource, AppFailure>> addVideo(NewVideoInput input) async {
    addCalls += 1;
    lastAddInput = input;
    return addResult ?? const Failure(ServerFailure(500));
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    addLocationCalls += 1;
    return addLocationResult ??
        Success(
          ResourceLocation(
            id: 'loc-new',
            resourceId: resourceId,
            deviceId: input.deviceId,
            pathOrUrl: input.pathOrUrl,
            storageType: input.storageType,
          ),
        );
  }

  @override
  Future<Result<void, AppFailure>> deleteVideo(String id) async => deleteResult;

  @override
  Future<Result<VideoDetail, AppFailure>> getVideo(String id) async {
    return detailResult ?? const Failure(NotFoundFailure('missing'));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listVideos() async {
    listCalls += 1;
    return listResult;
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(
    String resourceId,
    String locationId,
  ) async => removeLocationResult;

  @override
  Future<Result<List<Resource>, AppFailure>> searchVideos(String query) async => searchResult;

  @override
  Future<Result<Resource, AppFailure>> updateVideo(
    String id,
    UpdateVideoInput input,
  ) async => updateResult ?? const Failure(ServerFailure(500));
}

class FakeGameRepository implements GameRepository {
  FakeGameRepository({
    this.listResult = const Success([]),
    this.searchResult = const Success([]),
    this.detailResult,
    this.addResult,
    this.updateResult,
    this.deleteResult = const Success(null),
    this.addLocationResult,
    this.removeLocationResult = const Success(null),
  });

  Result<List<Resource>, AppFailure> listResult;
  Result<List<Resource>, AppFailure> searchResult;
  Result<GameDetail, AppFailure>? detailResult;
  Result<Resource, AppFailure>? addResult;
  Result<Resource, AppFailure>? updateResult;
  Result<void, AppFailure> deleteResult;
  Result<ResourceLocation, AppFailure>? addLocationResult;
  Result<void, AppFailure> removeLocationResult;

  int listCalls = 0;
  int addCalls = 0;
  int updateCalls = 0;
  int addLocationCalls = 0;
  NewGameInput? lastAddInput;

  @override
  Future<Result<Resource, AppFailure>> addGame(NewGameInput input) async {
    addCalls += 1;
    lastAddInput = input;
    return addResult ?? const Failure(ServerFailure(500));
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) async {
    addLocationCalls += 1;
    return addLocationResult ??
        Success(
          ResourceLocation(
            id: 'loc-new',
            resourceId: resourceId,
            deviceId: input.deviceId,
            pathOrUrl: input.pathOrUrl,
            storageType: input.storageType,
          ),
        );
  }

  @override
  Future<Result<void, AppFailure>> deleteGame(String id) async => deleteResult;

  @override
  Future<Result<GameDetail, AppFailure>> getGame(String id) async {
    return detailResult ?? const Failure(NotFoundFailure('missing'));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listGames() async {
    listCalls += 1;
    return listResult;
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(
    String resourceId,
    String locationId,
  ) async => removeLocationResult;

  @override
  Future<Result<List<Resource>, AppFailure>> searchGames(String query) async => searchResult;

  @override
  Future<Result<Resource, AppFailure>> updateGame(
    String id,
    UpdateGameInput input,
  ) async => updateResult ?? const Failure(ServerFailure(500));
}

class FakeVaultRepository implements VaultRepository {
  FakeVaultRepository({
    this.statusResult = const Success(
      VaultStatus(initialized: false, unlocked: false),
    ),
    this.initializeResult = const Success(null),
    this.unlockResult = const Success(null),
    this.lockResult = const Success(null),
    this.storeCredentialResult = const Success(null),
    this.retrieveCredentialResult = const Success('plaintext'),
    this.deleteCredentialResult = const Success(null),
    this.listPlatformsResult = const Success([]),
  });

  Result<VaultStatus, AppFailure> statusResult;
  Result<void, AppFailure> initializeResult;
  Result<void, AppFailure> unlockResult;
  Result<void, AppFailure> lockResult;
  Result<void, AppFailure> storeCredentialResult;
  Result<String, AppFailure> retrieveCredentialResult;
  Result<void, AppFailure> deleteCredentialResult;
  Result<List<String>, AppFailure> listPlatformsResult;

  int statusCalls = 0;
  int initializeCalls = 0;
  int unlockCalls = 0;
  int lockCalls = 0;
  int storeCredentialCalls = 0;
  int retrieveCredentialCalls = 0;
  int deleteCredentialCalls = 0;
  int listPlatformsCalls = 0;
  String? lastMasterPassword;
  StoreCredentialInput? lastStoreInput;
  RetrieveCredentialInput? lastRetrieveInput;

  @override
  Future<Result<VaultStatus, AppFailure>> getStatus() async {
    statusCalls += 1;
    return statusResult;
  }

  @override
  Future<Result<void, AppFailure>> initialize(String masterPassword) async {
    initializeCalls += 1;
    lastMasterPassword = masterPassword;
    return initializeResult;
  }

  @override
  Future<Result<void, AppFailure>> unlock(String masterPassword) async {
    unlockCalls += 1;
    lastMasterPassword = masterPassword;
    return unlockResult;
  }

  @override
  Future<Result<void, AppFailure>> lock() async {
    lockCalls += 1;
    return lockResult;
  }

  @override
  Future<Result<void, AppFailure>> storeCredential(StoreCredentialInput input) async {
    storeCredentialCalls += 1;
    lastStoreInput = input;
    return storeCredentialResult;
  }

  @override
  Future<Result<String, AppFailure>> retrieveCredential(RetrieveCredentialInput input) async {
    retrieveCredentialCalls += 1;
    lastRetrieveInput = input;
    return retrieveCredentialResult;
  }

  @override
  Future<Result<void, AppFailure>> deleteCredential(RetrieveCredentialInput input) async {
    deleteCredentialCalls += 1;
    return deleteCredentialResult;
  }

  @override
  Future<Result<List<String>, AppFailure>> listPlatforms() async {
    listPlatformsCalls += 1;
    return listPlatformsResult;
  }
}

class FakeDedupRepository implements DedupRepository {
  FakeDedupRepository({
    this.scanResult = const Success([]),
    this.listPendingResult = const Success([]),
    this.dismissResult = const Success(null),
    this.mergeResult = const Success(null),
  });

  Result<List<DedupWarning>, AppFailure> scanResult;
  Result<List<DedupWarning>, AppFailure> listPendingResult;
  Result<void, AppFailure> dismissResult;
  Result<void, AppFailure> mergeResult;

  int scanCalls = 0;
  int listPendingCalls = 0;
  int dismissCalls = 0;
  int mergeCalls = 0;
  String? lastDismissId;
  String? lastMergeWarningId;
  MergeInput? lastMergeInput;

  @override
  Future<Result<List<DedupWarning>, AppFailure>> scanDuplicates() async {
    scanCalls += 1;
    return scanResult;
  }

  @override
  Future<Result<List<DedupWarning>, AppFailure>> listPendingWarnings() async {
    listPendingCalls += 1;
    return listPendingResult;
  }

  @override
  Future<Result<void, AppFailure>> dismissWarning(String id) async {
    dismissCalls += 1;
    lastDismissId = id;
    return dismissResult;
  }

  @override
  Future<Result<void, AppFailure>> mergeResources(String warningId, MergeInput input) async {
    mergeCalls += 1;
    lastMergeWarningId = warningId;
    lastMergeInput = input;
    return mergeResult;
  }
}

class FakeDeviceRepository implements DeviceRepository {
  FakeDeviceRepository({
    this.listResult = const Success([]),
    Result<Device, AppFailure>? currentResult,
    Result<Device, AppFailure>? registerResult,
    this.delinkResult = const Success(null),
  })  : currentResult = currentResult ??
            const Failure(NotFoundFailure('current device')),
        registerResult = registerResult ??
            const Failure(ServerFailure(500));

  Result<List<Device>, AppFailure> listResult;
  Result<Device, AppFailure> currentResult;
  Result<Device, AppFailure> registerResult;
  Result<void, AppFailure> delinkResult;

  int listCalls = 0;
  int currentCalls = 0;
  int registerCalls = 0;
  String? lastDelinkId;
  String? lastRegisterDeviceId;
  String? lastRegisterDeviceName;

  @override
  Future<Result<List<Device>, AppFailure>> listDevices() async {
    listCalls += 1;
    return listResult;
  }

  @override
  Future<Result<Device, AppFailure>> currentDevice() async {
    currentCalls += 1;
    return currentResult;
  }

  @override
  Future<Result<Device, AppFailure>> registerDevice({
    required String deviceId,
    String? deviceName,
  }) async {
    registerCalls += 1;
    lastRegisterDeviceId = deviceId;
    lastRegisterDeviceName = deviceName;
    return registerResult;
  }

  @override
  Future<Result<void, AppFailure>> delinkDevice(String deviceId) async {
    lastDelinkId = deviceId;
    return delinkResult;
  }
}

class FakeBatchOperationRepository implements BatchOperationRepository {
  FakeBatchOperationRepository({
    this.importResult = const Success(BatchOperationResponse(
      type: BatchOperationType.importResources,
      results: [],
    )),
    this.updateResult = const Success(BatchOperationResponse(
      type: BatchOperationType.updateMetadata,
      results: [],
    )),
    this.copyResult = const Success(BatchOperationResponse(
      type: BatchOperationType.copyMetadata,
      results: [],
    )),
  });

  Result<BatchOperationResponse, AppFailure> importResult;
  Result<BatchOperationResponse, AppFailure> updateResult;
  Result<BatchOperationResponse, AppFailure> copyResult;

  int importCalls = 0;
  int updateCalls = 0;
  int copyCalls = 0;
  BatchImportRequest? lastImportRequest;
  BatchMetadataUpdateRequest? lastUpdateRequest;
  BatchMetadataCopyRequest? lastCopyRequest;

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchImport(
    BatchImportRequest request,
  ) async {
    importCalls += 1;
    lastImportRequest = request;
    return importResult;
  }

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchUpdateMetadata(
    BatchMetadataUpdateRequest request,
  ) async {
    updateCalls += 1;
    lastUpdateRequest = request;
    return updateResult;
  }

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchCopyMetadata(
    BatchMetadataCopyRequest request,
  ) async {
    copyCalls += 1;
    lastCopyRequest = request;
    return copyResult;
  }
}
