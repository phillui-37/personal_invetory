import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/repository_inputs.dart';
import '../../models/resources.dart';
import '../../models/result.dart';
import '../../repositories/game_repository.dart';

sealed class GameEvent extends Equatable {
  const GameEvent();

  @override
  List<Object?> get props => [];
}

final class LoadGames extends GameEvent {
  const LoadGames({
    this.tags = const [],
    this.sortBy,
    this.sortOrder,
    this.filterLogic,
  });

  final List<String> tags;
  final String? sortBy;
  final String? sortOrder;
  final String? filterLogic;

  @override
  List<Object?> get props => [tags, sortBy, sortOrder, filterLogic];
}

final class SearchGames extends GameEvent {
  const SearchGames(this.query);

  final String query;

  @override
  List<Object?> get props => [query];
}

final class AddGame extends GameEvent {
  const AddGame(this.input);

  final NewGameInput input;

  @override
  List<Object?> get props => [input];
}

final class UpdateGame extends GameEvent {
  const UpdateGame(this.id, this.input);

  final String id;
  final UpdateGameInput input;

  @override
  List<Object?> get props => [id, input];
}

final class DeleteGame extends GameEvent {
  const DeleteGame(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class LoadGameDetail extends GameEvent {
  const LoadGameDetail(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class AddGameLocation extends GameEvent {
  const AddGameLocation(this.resourceId, this.input);

  final String resourceId;
  final NewLocationInput input;

  @override
  List<Object?> get props => [resourceId, input];
}

final class RemoveGameLocation extends GameEvent {
  const RemoveGameLocation(this.resourceId, this.locationId);

  final String resourceId;
  final String locationId;

  @override
  List<Object?> get props => [resourceId, locationId];
}

sealed class GameState extends Equatable {
  const GameState();

  @override
  List<Object?> get props => [];
}

final class GameInitial extends GameState {
  const GameInitial();
}

final class GameLoading extends GameState {
  const GameLoading();
}

final class GameListLoaded extends GameState {
  const GameListLoaded(this.games);

  final List<Resource> games;

  @override
  List<Object?> get props => [games];
}

final class GameDetailLoaded extends GameState {
  const GameDetailLoaded(this.game);

  final GameDetail game;

  @override
  List<Object?> get props => [game];
}

enum GameOperationType {
  added,
  updated,
  deleted,
  locationAdded,
  locationRemoved,
}

final class GameOperationSuccess extends GameState {
  const GameOperationSuccess(this.operationType);

  final GameOperationType operationType;

  @override
  List<Object?> get props => [operationType];
}

final class GameError extends GameState {
  const GameError(this.failure);

  final AppFailure failure;

  @override
  List<Object?> get props => [failure];
}

final class GameBloc extends Bloc<GameEvent, GameState> {
  GameBloc(this._repository) : super(const GameInitial()) {
    on<LoadGames>(_onLoadGames);
    on<SearchGames>(_onSearchGames);
    on<LoadGameDetail>(_onLoadGameDetail);
    on<AddGame>(_onAddGame);
    on<UpdateGame>(_onUpdateGame);
    on<DeleteGame>(_onDeleteGame);
    on<AddGameLocation>(_onAddGameLocation);
    on<RemoveGameLocation>(_onRemoveGameLocation);
  }

  final GameRepository _repository;

  Future<void> _onLoadGames(LoadGames event, Emitter<GameState> emit) async {
    emit(const GameLoading());
    final result = await _repository.listGames(
      tags: event.tags,
      sortBy: event.sortBy,
      sortOrder: event.sortOrder,
      filterLogic: event.filterLogic,
    );
    result.when(
      success: (games) => emit(GameListLoaded(games)),
      failure: (failure) => emit(GameError(failure)),
    );
  }

  Future<void> _onSearchGames(SearchGames event, Emitter<GameState> emit) async {
    emit(const GameLoading());
    final result = await _repository.searchGames(event.query);
    result.when(
      success: (games) => emit(GameListLoaded(games)),
      failure: (failure) => emit(GameError(failure)),
    );
  }

  Future<void> _onLoadGameDetail(
    LoadGameDetail event,
    Emitter<GameState> emit,
  ) async {
    emit(const GameLoading());
    final result = await _repository.getGame(event.id);
    result.when(
      success: (game) => emit(GameDetailLoaded(game)),
      failure: (failure) => emit(GameError(failure)),
    );
  }

  Future<void> _onAddGame(AddGame event, Emitter<GameState> emit) async {
    emit(const GameLoading());
    final result = await _repository.addGame(event.input);
    _emitOperationResult(result, emit, GameOperationType.added);
  }

  Future<void> _onUpdateGame(UpdateGame event, Emitter<GameState> emit) async {
    emit(const GameLoading());
    final result = await _repository.updateGame(event.id, event.input);
    _emitOperationResult(result, emit, GameOperationType.updated);
  }

  Future<void> _onDeleteGame(DeleteGame event, Emitter<GameState> emit) async {
    emit(const GameLoading());
    final result = await _repository.deleteGame(event.id);
    _emitVoidOperationResult(result, emit, GameOperationType.deleted);
  }

  Future<void> _onAddGameLocation(
    AddGameLocation event,
    Emitter<GameState> emit,
  ) async {
    emit(const GameLoading());
    final result = await _repository.addLocation(event.resourceId, event.input);
    _emitLocationOperationResult(result, emit, GameOperationType.locationAdded);
  }

  Future<void> _onRemoveGameLocation(
    RemoveGameLocation event,
    Emitter<GameState> emit,
  ) async {
    emit(const GameLoading());
    final result = await _repository.removeLocation(
      event.resourceId,
      event.locationId,
    );
    _emitVoidOperationResult(result, emit, GameOperationType.locationRemoved);
  }

  void _emitOperationResult(
    Result<Resource, AppFailure> result,
    Emitter<GameState> emit,
    GameOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(GameOperationSuccess(operationType)),
      failure: (failure) => emit(GameError(failure)),
    );
  }

  void _emitLocationOperationResult(
    Result<ResourceLocation, AppFailure> result,
    Emitter<GameState> emit,
    GameOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(GameOperationSuccess(operationType)),
      failure: (failure) => emit(GameError(failure)),
    );
  }

  void _emitVoidOperationResult(
    Result<void, AppFailure> result,
    Emitter<GameState> emit,
    GameOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(GameOperationSuccess(operationType)),
      failure: (failure) => emit(GameError(failure)),
    );
  }
}
