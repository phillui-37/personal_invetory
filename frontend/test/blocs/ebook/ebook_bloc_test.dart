import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/ebook/ebook_bloc.dart';
import 'package:personal_inventory_frontend/models/batch_operations.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/repository_inputs.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/ebook_repository.dart';

void main() {
  group('EbookBloc', () {
    blocTest<EbookBloc, EbookState>(
      'emits loading then list loaded when LoadEbooks succeeds',
      build: () => EbookBloc(_FakeEbookRepository(
        onListEbooks: () async => const Success([
          Resource(id: 'e1', title: 'Book 1', resourceType: ResourceType.ebook),
        ]),
      )),
      act: (bloc) => bloc.add(const LoadEbooks()),
      expect: () => const [
        EbookLoading(),
        EbookListLoaded([
          Resource(id: 'e1', title: 'Book 1', resourceType: ResourceType.ebook),
        ]),
      ],
    );

    blocTest<EbookBloc, EbookState>(
      'emits loading then detail loaded when LoadEbookDetail succeeds',
      build: () => EbookBloc(_FakeEbookRepository(
        onGetEbook: (_) async => const Success(
          EbookDetail(
            resource: Resource(
              id: 'e1',
              title: 'Book 1',
              resourceType: ResourceType.ebook,
            ),
            meta: EbookMeta(resourceId: 'e1', author: 'A'),
            locations: [],
          ),
        ),
      )),
      act: (bloc) => bloc.add(const LoadEbookDetail('e1')),
      expect: () => const [
        EbookLoading(),
        EbookDetailLoaded(
          EbookDetail(
            resource: Resource(
              id: 'e1',
              title: 'Book 1',
              resourceType: ResourceType.ebook,
            ),
            meta: EbookMeta(resourceId: 'e1', author: 'A'),
            locations: [],
          ),
        ),
      ],
    );

    blocTest<EbookBloc, EbookState>(
      'emits loading then error when LoadEbooks fails',
      build: () => EbookBloc(_FakeEbookRepository(
        onListEbooks: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const LoadEbooks()),
      expect: () => const [
        EbookLoading(),
        EbookError(NetworkFailure('offline')),
      ],
    );

    blocTest<EbookBloc, EbookState>(
      'emits loading then operation success when AddEbook succeeds',
      build: () => EbookBloc(_FakeEbookRepository(
        onAddEbook: (_) async => const Success(
          Resource(id: 'e2', title: 'Book 2', resourceType: ResourceType.ebook),
        ),
      )),
      act: (bloc) => bloc.add(
        const AddEbook(
          NewEbookInput(
            resource: Resource(
              id: 'e2',
              title: 'Book 2',
              resourceType: ResourceType.ebook,
            ),
            meta: EbookMeta(resourceId: 'e2'),
          ),
        ),
      ),
      expect: () => const [
        EbookLoading(),
        EbookOperationSuccess(EbookOperationType.added),
      ],
    );
  });
}

final class _FakeEbookRepository implements EbookRepository {
  _FakeEbookRepository({
    Future<Result<List<Resource>, AppFailure>> Function()? onListEbooks,
    Future<Result<List<Resource>, AppFailure>> Function(String query)? onSearchEbooks,
    Future<Result<EbookDetail, AppFailure>> Function(String id)? onGetEbook,
    Future<Result<Resource, AppFailure>> Function(NewEbookInput input)? onAddEbook,
    Future<Result<Resource, AppFailure>> Function(String id, UpdateEbookInput input)?
        onUpdateEbook,
    Future<Result<void, AppFailure>> Function(String id)? onDeleteEbook,
    Future<Result<ResourceLocation, AppFailure>> Function(
      String resourceId,
      NewLocationInput input,
    )? onAddLocation,
    Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
        onRemoveLocation,
  }) : _onListEbooks = onListEbooks,
       _onSearchEbooks = onSearchEbooks,
       _onGetEbook = onGetEbook,
       _onAddEbook = onAddEbook,
       _onUpdateEbook = onUpdateEbook,
       _onDeleteEbook = onDeleteEbook,
       _onAddLocation = onAddLocation,
       _onRemoveLocation = onRemoveLocation;

  final Future<Result<List<Resource>, AppFailure>> Function()? _onListEbooks;
  final Future<Result<List<Resource>, AppFailure>> Function(String query)? _onSearchEbooks;
  final Future<Result<EbookDetail, AppFailure>> Function(String id)? _onGetEbook;
  final Future<Result<Resource, AppFailure>> Function(NewEbookInput input)? _onAddEbook;
  final Future<Result<Resource, AppFailure>> Function(String id, UpdateEbookInput input)?
      _onUpdateEbook;
  final Future<Result<void, AppFailure>> Function(String id)? _onDeleteEbook;
  final Future<Result<ResourceLocation, AppFailure>> Function(
    String resourceId,
    NewLocationInput input,
  )?
  _onAddLocation;
  final Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
      _onRemoveLocation;

  @override
  Future<Result<Resource, AppFailure>> addEbook(NewEbookInput input) {
    return _onAddEbook?.call(input) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  ) {
    return _onAddLocation?.call(resourceId, input) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<void, AppFailure>> deleteEbook(String id) {
    return _onDeleteEbook?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<EbookDetail, AppFailure>> getEbook(String id) {
    return _onGetEbook?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listEbooks({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    return _onListEbooks?.call() ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) {
    return _onRemoveLocation?.call(resourceId, locationId) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchEbooks(String query) {
    return _onSearchEbooks?.call(query) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<Resource, AppFailure>> updateEbook(String id, UpdateEbookInput input) {
    return _onUpdateEbook?.call(id, input) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<BatchImportResult, AppFailure>> batchImport(
    List<BatchImportEntry> entries,
  ) {
    return Future.value(const Success(BatchImportResult(succeeded: [], failed: [])));
  }
}
