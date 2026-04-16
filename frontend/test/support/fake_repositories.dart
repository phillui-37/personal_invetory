import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/repository_inputs.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/ebook_repository.dart';
import 'package:personal_inventory_frontend/repositories/web_reader_repository.dart';

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
  });

  Result<List<Resource>, AppFailure> listResult;
  Result<List<Resource>, AppFailure> searchResult;
  Result<EbookDetail, AppFailure>? detailResult;
  Result<Resource, AppFailure>? addResult;
  Result<Resource, AppFailure>? updateResult;
  Result<void, AppFailure> deleteResult;
  Result<ResourceLocation, AppFailure>? addLocationResult;
  Result<void, AppFailure> removeLocationResult;

  int listCalls = 0;
  int searchCalls = 0;
  int deleteCalls = 0;
  int addCalls = 0;
  int updateCalls = 0;
  int addLocationCalls = 0;
  String? lastUpdateId;
  UpdateEbookInput? lastUpdateInput;
  String? lastAddLocationResourceId;
  NewLocationInput? lastAddLocationInput;
  NewEbookInput? lastAddInput;

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

  int listCalls = 0;
  int searchCalls = 0;
  int trackProgressCalls = 0;
  int deleteCalls = 0;
  int addCalls = 0;
  int updateCalls = 0;
  int addLocationCalls = 0;
  String? lastUpdateId;
  UpdateWebReaderInput? lastUpdateInput;
  String? lastAddLocationResourceId;
  NewLocationInput? lastAddLocationInput;
  NewWebReaderInput? lastAddInput;

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
