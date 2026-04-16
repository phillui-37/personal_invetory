import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../models/result.dart';

abstract interface class WebReaderRepository {
  Future<Result<List<Resource>, AppFailure>> listWebReaders();

  Future<Result<List<Resource>, AppFailure>> searchWebReaders(String query);

  Future<Result<WebReaderDetail, AppFailure>> getWebReader(String id);

  Future<Result<Resource, AppFailure>> addWebReader(NewWebReaderInput input);

  Future<Result<Resource, AppFailure>> updateWebReader(
    String id,
    UpdateWebReaderInput input,
  );

  Future<Result<void, AppFailure>> deleteWebReader(String id);

  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  );

  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId);

  Future<Result<void, AppFailure>> trackProgress(WebReaderProgressSignal signal);
}
