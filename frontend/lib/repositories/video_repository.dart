import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../models/result.dart';

abstract interface class VideoRepository {
  Future<Result<List<Resource>, AppFailure>> listVideos();

  Future<Result<List<Resource>, AppFailure>> searchVideos(String query);

  Future<Result<VideoDetail, AppFailure>> getVideo(String id);

  Future<Result<Resource, AppFailure>> addVideo(NewVideoInput input);

  Future<Result<Resource, AppFailure>> updateVideo(String id, UpdateVideoInput input);

  Future<Result<void, AppFailure>> deleteVideo(String id);

  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  );

  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId);
}
