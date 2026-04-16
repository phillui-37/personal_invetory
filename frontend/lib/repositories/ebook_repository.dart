import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../models/result.dart';

abstract interface class EbookRepository {
  Future<Result<List<Resource>, AppFailure>> listEbooks();

  Future<Result<List<Resource>, AppFailure>> searchEbooks(String query);

  Future<Result<EbookDetail, AppFailure>> getEbook(String id);

  Future<Result<Resource, AppFailure>> addEbook(NewEbookInput input);

  Future<Result<Resource, AppFailure>> updateEbook(String id, UpdateEbookInput input);

  Future<Result<void, AppFailure>> deleteEbook(String id);

  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  );

  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId);
}
