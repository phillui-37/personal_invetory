import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/video/video_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/repository_inputs.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/video_repository.dart';

void main() {
  group('VideoBloc', () {
    blocTest<VideoBloc, VideoState>(
      'emits loading then list loaded when LoadVideos succeeds',
      build: () => VideoBloc(_FakeVideoRepository(
        onListVideos: () async => const Success([
          Resource(id: 'v1', title: 'Video 1', resourceType: ResourceType.video),
        ]),
      )),
      act: (bloc) => bloc.add(const LoadVideos()),
      expect: () => const [
        VideoLoading(),
        VideoListLoaded([
          Resource(id: 'v1', title: 'Video 1', resourceType: ResourceType.video),
        ]),
      ],
    );

    blocTest<VideoBloc, VideoState>(
      'emits loading then detail loaded when LoadVideoDetail succeeds',
      build: () => VideoBloc(_FakeVideoRepository(
        onGetVideo: (_) async => const Success(
          VideoDetail(
            resource: Resource(
              id: 'v1',
              title: 'Video 1',
              resourceType: ResourceType.video,
            ),
            meta: VideoMeta(resourceId: 'v1', durationSecs: 120),
            locations: [],
          ),
        ),
      )),
      act: (bloc) => bloc.add(const LoadVideoDetail('v1')),
      expect: () => const [
        VideoLoading(),
        VideoDetailLoaded(
          VideoDetail(
            resource: Resource(
              id: 'v1',
              title: 'Video 1',
              resourceType: ResourceType.video,
            ),
            meta: VideoMeta(resourceId: 'v1', durationSecs: 120),
            locations: [],
          ),
        ),
      ],
    );

    blocTest<VideoBloc, VideoState>(
      'emits loading then error when LoadVideos fails',
      build: () => VideoBloc(_FakeVideoRepository(
        onListVideos: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const LoadVideos()),
      expect: () => const [
        VideoLoading(),
        VideoError(NetworkFailure('offline')),
      ],
    );

    blocTest<VideoBloc, VideoState>(
      'emits loading then operation success when AddVideo succeeds',
      build: () => VideoBloc(_FakeVideoRepository(
        onAddVideo: (_) async => const Success(
          Resource(id: 'v2', title: 'Video 2', resourceType: ResourceType.video),
        ),
      )),
      act: (bloc) => bloc.add(
        const AddVideo(
          NewVideoInput(
            resource: Resource(
              id: 'v2',
              title: 'Video 2',
              resourceType: ResourceType.video,
            ),
            meta: VideoMeta(resourceId: 'v2'),
          ),
        ),
      ),
      expect: () => const [
        VideoLoading(),
        VideoOperationSuccess(VideoOperationType.added),
      ],
    );
  });
}

final class _FakeVideoRepository implements VideoRepository {
  _FakeVideoRepository({
    Future<Result<List<Resource>, AppFailure>> Function()? onListVideos,
    Future<Result<List<Resource>, AppFailure>> Function(String query)? onSearchVideos,
    Future<Result<VideoDetail, AppFailure>> Function(String id)? onGetVideo,
    Future<Result<Resource, AppFailure>> Function(NewVideoInput input)? onAddVideo,
    Future<Result<Resource, AppFailure>> Function(String id, UpdateVideoInput input)?
        onUpdateVideo,
    Future<Result<void, AppFailure>> Function(String id)? onDeleteVideo,
    Future<Result<ResourceLocation, AppFailure>> Function(
      String resourceId,
      NewLocationInput input,
    )? onAddLocation,
    Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
        onRemoveLocation,
  }) : _onListVideos = onListVideos,
       _onSearchVideos = onSearchVideos,
       _onGetVideo = onGetVideo,
       _onAddVideo = onAddVideo,
       _onUpdateVideo = onUpdateVideo,
       _onDeleteVideo = onDeleteVideo,
       _onAddLocation = onAddLocation,
       _onRemoveLocation = onRemoveLocation;

  final Future<Result<List<Resource>, AppFailure>> Function()? _onListVideos;
  final Future<Result<List<Resource>, AppFailure>> Function(String query)? _onSearchVideos;
  final Future<Result<VideoDetail, AppFailure>> Function(String id)? _onGetVideo;
  final Future<Result<Resource, AppFailure>> Function(NewVideoInput input)? _onAddVideo;
  final Future<Result<Resource, AppFailure>> Function(String id, UpdateVideoInput input)?
      _onUpdateVideo;
  final Future<Result<void, AppFailure>> Function(String id)? _onDeleteVideo;
  final Future<Result<ResourceLocation, AppFailure>> Function(
    String resourceId,
    NewLocationInput input,
  )? _onAddLocation;
  final Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
      _onRemoveLocation;

  @override
  Future<Result<Resource, AppFailure>> addVideo(NewVideoInput input) {
    return _onAddVideo?.call(input) ??
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
  Future<Result<void, AppFailure>> deleteVideo(String id) {
    return _onDeleteVideo?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<VideoDetail, AppFailure>> getVideo(String id) {
    return _onGetVideo?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listVideos() {
    return _onListVideos?.call() ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) {
    return _onRemoveLocation?.call(resourceId, locationId) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchVideos(String query) {
    return _onSearchVideos?.call(query) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<Resource, AppFailure>> updateVideo(String id, UpdateVideoInput input) {
    return _onUpdateVideo?.call(id, input) ??
        Future.value(const Failure(ServerFailure(500)));
  }
}
