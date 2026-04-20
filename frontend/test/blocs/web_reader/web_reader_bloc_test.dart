import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/web_reader/web_reader_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/repository_inputs.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/web_reader_repository.dart';

void main() {
  group('WebReaderBloc', () {
    blocTest<WebReaderBloc, WebReaderState>(
      'emits loading then list loaded when LoadWebReaders succeeds',
      build: () => WebReaderBloc(_FakeWebReaderRepository(
        onListWebReaders: () async => const Success([
          Resource(
            id: 'w1',
            title: 'Reader 1',
            resourceType: ResourceType.webReader,
          ),
        ]),
      )),
      act: (bloc) => bloc.add(const LoadWebReaders()),
      expect: () => const [
        WebReaderLoading(),
        WebReaderListLoaded([
          Resource(
            id: 'w1',
            title: 'Reader 1',
            resourceType: ResourceType.webReader,
          ),
        ]),
      ],
    );

    blocTest<WebReaderBloc, WebReaderState>(
      'emits loading then operation success when TrackWebReaderProgress succeeds',
      build: () => WebReaderBloc(_FakeWebReaderRepository(
        onTrackProgress: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(
        const TrackWebReaderProgress(
          WebReaderProgressSignal(
            resourceId: 'w1',
            url: 'https://example.com/ch1',
            domProgress: 0.25,
          ),
        ),
      ),
      expect: () => const [
        WebReaderLoading(),
        WebReaderOperationSuccess(WebReaderOperationType.progressTracked),
      ],
    );

    final _checkedAt = DateTime.utc(2025, 1, 1);
    final _check = ChapterCheck(
      id: 'chk1',
      resourceId: 'w1',
      hasNewChapter: true,
      latestChapter: 'Chapter 42',
      checkedAt: _checkedAt,
    );

    blocTest<WebReaderBloc, WebReaderState>(
      'TriggerChapterCheck → emits loading then ChapterCheckTriggered on success',
      build: () => WebReaderBloc(_FakeWebReaderRepository(
        onTriggerCheck: (_) async => Success(_check),
      )),
      act: (bloc) => bloc.add(const TriggerChapterCheck('w1')),
      expect: () => [const WebReaderLoading(), ChapterCheckTriggered(_check)],
    );

    blocTest<WebReaderBloc, WebReaderState>(
      'TriggerChapterCheck → emits loading then WebReaderError on failure',
      build: () => WebReaderBloc(_FakeWebReaderRepository(
        onTriggerCheck: (_) async =>
            const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const TriggerChapterCheck('w1')),
      expect: () => const [
        WebReaderLoading(),
        WebReaderError(ServerFailure(500)),
      ],
    );

    blocTest<WebReaderBloc, WebReaderState>(
      'LoadCheckHistory → emits loading then CheckHistoryLoaded on success',
      build: () => WebReaderBloc(_FakeWebReaderRepository(
        onListCheckHistory: (_) async => Success([_check]),
      )),
      act: (bloc) => bloc.add(const LoadCheckHistory('w1')),
      expect: () => [const WebReaderLoading(), CheckHistoryLoaded([_check])],
    );
  });
}

final class _FakeWebReaderRepository implements WebReaderRepository {
  _FakeWebReaderRepository({
    Future<Result<List<Resource>, AppFailure>> Function()? onListWebReaders,
    Future<Result<List<Resource>, AppFailure>> Function(String query)? onSearchWebReaders,
    Future<Result<WebReaderDetail, AppFailure>> Function(String id)? onGetWebReader,
    Future<Result<Resource, AppFailure>> Function(NewWebReaderInput input)?
        onAddWebReader,
    Future<Result<Resource, AppFailure>> Function(
      String id,
      UpdateWebReaderInput input,
    )?
    onUpdateWebReader,
    Future<Result<void, AppFailure>> Function(String id)? onDeleteWebReader,
    Future<Result<ResourceLocation, AppFailure>> Function(
      String resourceId,
      NewLocationInput input,
    )? onAddLocation,
    Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
        onRemoveLocation,
    Future<Result<void, AppFailure>> Function(WebReaderProgressSignal signal)?
        onTrackProgress,
    Future<Result<ChapterCheck, AppFailure>> Function(String resourceId)?
        onTriggerCheck,
    Future<Result<List<ChapterCheck>, AppFailure>> Function(String resourceId)?
        onListCheckHistory,
  }) : _onListWebReaders = onListWebReaders,
       _onSearchWebReaders = onSearchWebReaders,
       _onGetWebReader = onGetWebReader,
       _onAddWebReader = onAddWebReader,
       _onUpdateWebReader = onUpdateWebReader,
       _onDeleteWebReader = onDeleteWebReader,
       _onAddLocation = onAddLocation,
       _onRemoveLocation = onRemoveLocation,
       _onTrackProgress = onTrackProgress,
       _onTriggerCheck = onTriggerCheck,
       _onListCheckHistory = onListCheckHistory;

  final Future<Result<List<Resource>, AppFailure>> Function()? _onListWebReaders;
  final Future<Result<List<Resource>, AppFailure>> Function(String query)?
      _onSearchWebReaders;
  final Future<Result<WebReaderDetail, AppFailure>> Function(String id)? _onGetWebReader;
  final Future<Result<Resource, AppFailure>> Function(NewWebReaderInput input)?
      _onAddWebReader;
  final Future<Result<Resource, AppFailure>> Function(
    String id,
    UpdateWebReaderInput input,
  )?
  _onUpdateWebReader;
  final Future<Result<void, AppFailure>> Function(String id)? _onDeleteWebReader;
  final Future<Result<ResourceLocation, AppFailure>> Function(
    String resourceId,
    NewLocationInput input,
  )?
  _onAddLocation;
  final Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
      _onRemoveLocation;
  final Future<Result<void, AppFailure>> Function(WebReaderProgressSignal signal)?
      _onTrackProgress;
  final Future<Result<ChapterCheck, AppFailure>> Function(String resourceId)?
      _onTriggerCheck;
  final Future<Result<List<ChapterCheck>, AppFailure>> Function(String resourceId)?
      _onListCheckHistory;

  @override
  Future<Result<Resource, AppFailure>> addWebReader(NewWebReaderInput input) {
    return _onAddWebReader?.call(input) ??
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
  Future<Result<void, AppFailure>> deleteWebReader(String id) {
    return _onDeleteWebReader?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<WebReaderDetail, AppFailure>> getWebReader(String id) {
    return _onGetWebReader?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listWebReaders({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    return _onListWebReaders?.call() ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(
    String resourceId,
    String locationId,
  ) {
    return _onRemoveLocation?.call(resourceId, locationId) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchWebReaders(String query) {
    return _onSearchWebReaders?.call(query) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<void, AppFailure>> trackProgress(WebReaderProgressSignal signal) {
    return _onTrackProgress?.call(signal) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<Resource, AppFailure>> updateWebReader(
    String id,
    UpdateWebReaderInput input,
  ) {
    return _onUpdateWebReader?.call(id, input) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<ChapterCheck, AppFailure>> triggerCheck(String resourceId) {
    return _onTriggerCheck?.call(resourceId) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<ChapterCheck>, AppFailure>> listCheckHistory(
    String resourceId,
  ) {
    return _onListCheckHistory?.call(resourceId) ??
        Future.value(const Success([]));
  }
}

