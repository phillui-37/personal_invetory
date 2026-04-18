import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../models/result.dart';

abstract interface class ImageRepository {
  Future<Result<List<Resource>, AppFailure>> listImages();

  Future<Result<List<Resource>, AppFailure>> searchImages(String query);

  Future<Result<ImageDetail, AppFailure>> getImage(String id);

  Future<Result<Resource, AppFailure>> addImage(NewImageInput input);

  Future<Result<Resource, AppFailure>> updateImage(String id, UpdateImageInput input);

  Future<Result<void, AppFailure>> deleteImage(String id);

  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  );

  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId);
}
