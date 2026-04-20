import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/repository_inputs.dart';
import '../../models/resources.dart';
import '../../models/result.dart';
import '../../repositories/ebook_repository.dart';

sealed class EbookEvent extends Equatable {
  const EbookEvent();

  @override
  List<Object?> get props => [];
}

final class LoadEbooks extends EbookEvent {
  const LoadEbooks({
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

final class SearchEbooks extends EbookEvent {
  const SearchEbooks(this.query);

  final String query;

  @override
  List<Object?> get props => [query];
}

final class AddEbook extends EbookEvent {
  const AddEbook(this.input);

  final NewEbookInput input;

  @override
  List<Object?> get props => [input];
}

final class UpdateEbook extends EbookEvent {
  const UpdateEbook(this.id, this.input);

  final String id;
  final UpdateEbookInput input;

  @override
  List<Object?> get props => [id, input];
}

final class DeleteEbook extends EbookEvent {
  const DeleteEbook(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class LoadEbookDetail extends EbookEvent {
  const LoadEbookDetail(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class AddEbookLocation extends EbookEvent {
  const AddEbookLocation(this.resourceId, this.input);

  final String resourceId;
  final NewLocationInput input;

  @override
  List<Object?> get props => [resourceId, input];
}

final class RemoveEbookLocation extends EbookEvent {
  const RemoveEbookLocation(this.resourceId, this.locationId);

  final String resourceId;
  final String locationId;

  @override
  List<Object?> get props => [resourceId, locationId];
}

sealed class EbookState extends Equatable {
  const EbookState();

  @override
  List<Object?> get props => [];
}

final class EbookInitial extends EbookState {
  const EbookInitial();
}

final class EbookLoading extends EbookState {
  const EbookLoading();
}

final class EbookListLoaded extends EbookState {
  const EbookListLoaded(this.ebooks);

  final List<Resource> ebooks;

  @override
  List<Object?> get props => [ebooks];
}

final class EbookDetailLoaded extends EbookState {
  const EbookDetailLoaded(this.ebook);

  final EbookDetail ebook;

  @override
  List<Object?> get props => [ebook];
}

enum EbookOperationType {
  added,
  updated,
  deleted,
  locationAdded,
  locationRemoved,
}

final class EbookOperationSuccess extends EbookState {
  const EbookOperationSuccess(this.operationType);

  final EbookOperationType operationType;

  @override
  List<Object?> get props => [operationType];
}

final class EbookError extends EbookState {
  const EbookError(this.failure);

  final AppFailure failure;

  @override
  List<Object?> get props => [failure];
}

final class EbookBloc extends Bloc<EbookEvent, EbookState> {
  EbookBloc(this._repository) : super(const EbookInitial()) {
    on<LoadEbooks>(_onLoadEbooks);
    on<SearchEbooks>(_onSearchEbooks);
    on<LoadEbookDetail>(_onLoadEbookDetail);
    on<AddEbook>(_onAddEbook);
    on<UpdateEbook>(_onUpdateEbook);
    on<DeleteEbook>(_onDeleteEbook);
    on<AddEbookLocation>(_onAddEbookLocation);
    on<RemoveEbookLocation>(_onRemoveEbookLocation);
  }

  final EbookRepository _repository;

  Future<void> _onLoadEbooks(LoadEbooks event, Emitter<EbookState> emit) async {
    emit(const EbookLoading());
    final result = await _repository.listEbooks();
    result.when(
      success: (ebooks) => emit(EbookListLoaded(ebooks)),
      failure: (failure) => emit(EbookError(failure)),
    );
  }

  Future<void> _onSearchEbooks(SearchEbooks event, Emitter<EbookState> emit) async {
    emit(const EbookLoading());
    final result = await _repository.searchEbooks(event.query);
    result.when(
      success: (ebooks) => emit(EbookListLoaded(ebooks)),
      failure: (failure) => emit(EbookError(failure)),
    );
  }

  Future<void> _onLoadEbookDetail(
    LoadEbookDetail event,
    Emitter<EbookState> emit,
  ) async {
    emit(const EbookLoading());
    final result = await _repository.getEbook(event.id);
    result.when(
      success: (ebook) => emit(EbookDetailLoaded(ebook)),
      failure: (failure) => emit(EbookError(failure)),
    );
  }

  Future<void> _onAddEbook(AddEbook event, Emitter<EbookState> emit) async {
    emit(const EbookLoading());
    final result = await _repository.addEbook(event.input);
    _emitOperationResult(result, emit, EbookOperationType.added);
  }

  Future<void> _onUpdateEbook(UpdateEbook event, Emitter<EbookState> emit) async {
    emit(const EbookLoading());
    final result = await _repository.updateEbook(event.id, event.input);
    _emitOperationResult(result, emit, EbookOperationType.updated);
  }

  Future<void> _onDeleteEbook(DeleteEbook event, Emitter<EbookState> emit) async {
    emit(const EbookLoading());
    final result = await _repository.deleteEbook(event.id);
    _emitVoidOperationResult(result, emit, EbookOperationType.deleted);
  }

  Future<void> _onAddEbookLocation(
    AddEbookLocation event,
    Emitter<EbookState> emit,
  ) async {
    emit(const EbookLoading());
    final result = await _repository.addLocation(event.resourceId, event.input);
    _emitLocationOperationResult(result, emit, EbookOperationType.locationAdded);
  }

  Future<void> _onRemoveEbookLocation(
    RemoveEbookLocation event,
    Emitter<EbookState> emit,
  ) async {
    emit(const EbookLoading());
    final result = await _repository.removeLocation(
      event.resourceId,
      event.locationId,
    );
    _emitVoidOperationResult(result, emit, EbookOperationType.locationRemoved);
  }

  void _emitOperationResult(
    Result<Resource, AppFailure> result,
    Emitter<EbookState> emit,
    EbookOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(EbookOperationSuccess(operationType)),
      failure: (failure) => emit(EbookError(failure)),
    );
  }

  void _emitLocationOperationResult(
    Result<ResourceLocation, AppFailure> result,
    Emitter<EbookState> emit,
    EbookOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(EbookOperationSuccess(operationType)),
      failure: (failure) => emit(EbookError(failure)),
    );
  }

  void _emitVoidOperationResult(
    Result<void, AppFailure> result,
    Emitter<EbookState> emit,
    EbookOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(EbookOperationSuccess(operationType)),
      failure: (failure) => emit(EbookError(failure)),
    );
  }
}
