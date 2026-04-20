import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/game/game_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/repository_inputs.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/game_repository.dart';

void main() {
  group('GameBloc', () {
    blocTest<GameBloc, GameState>(
      'emits loading then list loaded when LoadGames succeeds',
      build: () => GameBloc(_FakeGameRepository(
        onListGames: () async => const Success([
          Resource(id: 'g1', title: 'Game 1', resourceType: ResourceType.game),
        ]),
      )),
      act: (bloc) => bloc.add(const LoadGames()),
      expect: () => const [
        GameLoading(),
        GameListLoaded([
          Resource(id: 'g1', title: 'Game 1', resourceType: ResourceType.game),
        ]),
      ],
    );

    blocTest<GameBloc, GameState>(
      'emits loading then detail loaded when LoadGameDetail succeeds',
      build: () => GameBloc(_FakeGameRepository(
        onGetGame: (_) async => const Success(
          GameDetail(
            resource: Resource(
              id: 'g1',
              title: 'Game 1',
              resourceType: ResourceType.game,
            ),
            meta: GameMeta(resourceId: 'g1', platform: 'PC'),
            locations: [],
          ),
        ),
      )),
      act: (bloc) => bloc.add(const LoadGameDetail('g1')),
      expect: () => const [
        GameLoading(),
        GameDetailLoaded(
          GameDetail(
            resource: Resource(
              id: 'g1',
              title: 'Game 1',
              resourceType: ResourceType.game,
            ),
            meta: GameMeta(resourceId: 'g1', platform: 'PC'),
            locations: [],
          ),
        ),
      ],
    );

    blocTest<GameBloc, GameState>(
      'emits loading then error when LoadGames fails',
      build: () => GameBloc(_FakeGameRepository(
        onListGames: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const LoadGames()),
      expect: () => const [
        GameLoading(),
        GameError(NetworkFailure('offline')),
      ],
    );

    blocTest<GameBloc, GameState>(
      'emits loading then operation success when AddGame succeeds',
      build: () => GameBloc(_FakeGameRepository(
        onAddGame: (_) async => const Success(
          Resource(id: 'g2', title: 'Game 2', resourceType: ResourceType.game),
        ),
      )),
      act: (bloc) => bloc.add(
        const AddGame(
          NewGameInput(
            resource: Resource(
              id: 'g2',
              title: 'Game 2',
              resourceType: ResourceType.game,
            ),
            meta: GameMeta(resourceId: 'g2'),
          ),
        ),
      ),
      expect: () => const [
        GameLoading(),
        GameOperationSuccess(GameOperationType.added),
      ],
    );
  });
}

final class _FakeGameRepository implements GameRepository {
  _FakeGameRepository({
    Future<Result<List<Resource>, AppFailure>> Function()? onListGames,
    Future<Result<List<Resource>, AppFailure>> Function(String query)? onSearchGames,
    Future<Result<GameDetail, AppFailure>> Function(String id)? onGetGame,
    Future<Result<Resource, AppFailure>> Function(NewGameInput input)? onAddGame,
    Future<Result<Resource, AppFailure>> Function(String id, UpdateGameInput input)?
        onUpdateGame,
    Future<Result<void, AppFailure>> Function(String id)? onDeleteGame,
    Future<Result<ResourceLocation, AppFailure>> Function(
      String resourceId,
      NewLocationInput input,
    )? onAddLocation,
    Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
        onRemoveLocation,
  }) : _onListGames = onListGames,
       _onSearchGames = onSearchGames,
       _onGetGame = onGetGame,
       _onAddGame = onAddGame,
       _onUpdateGame = onUpdateGame,
       _onDeleteGame = onDeleteGame,
       _onAddLocation = onAddLocation,
       _onRemoveLocation = onRemoveLocation;

  final Future<Result<List<Resource>, AppFailure>> Function()? _onListGames;
  final Future<Result<List<Resource>, AppFailure>> Function(String query)? _onSearchGames;
  final Future<Result<GameDetail, AppFailure>> Function(String id)? _onGetGame;
  final Future<Result<Resource, AppFailure>> Function(NewGameInput input)? _onAddGame;
  final Future<Result<Resource, AppFailure>> Function(String id, UpdateGameInput input)?
      _onUpdateGame;
  final Future<Result<void, AppFailure>> Function(String id)? _onDeleteGame;
  final Future<Result<ResourceLocation, AppFailure>> Function(
    String resourceId,
    NewLocationInput input,
  )? _onAddLocation;
  final Future<Result<void, AppFailure>> Function(String resourceId, String locationId)?
      _onRemoveLocation;

  @override
  Future<Result<Resource, AppFailure>> addGame(NewGameInput input) {
    return _onAddGame?.call(input) ??
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
  Future<Result<void, AppFailure>> deleteGame(String id) {
    return _onDeleteGame?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<GameDetail, AppFailure>> getGame(String id) {
    return _onGetGame?.call(id) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> listGames({
    List<String> tags = const [],
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    return _onListGames?.call() ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<void, AppFailure>> removeLocation(String resourceId, String locationId) {
    return _onRemoveLocation?.call(resourceId, locationId) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<List<Resource>, AppFailure>> searchGames(String query) {
    return _onSearchGames?.call(query) ??
        Future.value(const Failure(ServerFailure(500)));
  }

  @override
  Future<Result<Resource, AppFailure>> updateGame(String id, UpdateGameInput input) {
    return _onUpdateGame?.call(id, input) ??
        Future.value(const Failure(ServerFailure(500)));
  }
}
