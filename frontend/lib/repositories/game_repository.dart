import '../models/failures.dart';
import '../models/repository_inputs.dart';
import '../models/resources.dart';
import '../models/result.dart';

abstract interface class GameRepository {
  Future<Result<List<Resource>, AppFailure>> listGames({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  });

  Future<Result<List<Resource>, AppFailure>> searchGames(String query);

  Future<Result<GameDetail, AppFailure>> getGame(String id);

  Future<Result<Resource, AppFailure>> addGame(NewGameInput input);

  Future<Result<Resource, AppFailure>> updateGame(String id, UpdateGameInput input);

  Future<Result<void, AppFailure>> deleteGame(String id);

  Future<Result<ResourceLocation, AppFailure>> addLocation(
    String resourceId,
    NewLocationInput input,
  );

  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId);
}
