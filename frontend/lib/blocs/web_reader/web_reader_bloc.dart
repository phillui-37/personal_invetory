import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/repository_inputs.dart';
import '../../models/resources.dart';
import '../../models/result.dart';
import '../../repositories/web_reader_repository.dart';

sealed class WebReaderEvent extends Equatable {
  const WebReaderEvent();

  @override
  List<Object?> get props => [];
}

final class LoadWebReaders extends WebReaderEvent {
  const LoadWebReaders();
}

final class SearchWebReaders extends WebReaderEvent {
  const SearchWebReaders(this.query);

  final String query;

  @override
  List<Object?> get props => [query];
}

final class AddWebReader extends WebReaderEvent {
  const AddWebReader(this.input);

  final NewWebReaderInput input;

  @override
  List<Object?> get props => [input];
}

final class UpdateWebReader extends WebReaderEvent {
  const UpdateWebReader(this.id, this.input);

  final String id;
  final UpdateWebReaderInput input;

  @override
  List<Object?> get props => [id, input];
}

final class DeleteWebReader extends WebReaderEvent {
  const DeleteWebReader(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class LoadWebReaderDetail extends WebReaderEvent {
  const LoadWebReaderDetail(this.id);

  final String id;

  @override
  List<Object?> get props => [id];
}

final class AddWebReaderLocation extends WebReaderEvent {
  const AddWebReaderLocation(this.resourceId, this.input);

  final String resourceId;
  final NewLocationInput input;

  @override
  List<Object?> get props => [resourceId, input];
}

final class RemoveWebReaderLocation extends WebReaderEvent {
  const RemoveWebReaderLocation(this.resourceId, this.locationId);

  final String resourceId;
  final String locationId;

  @override
  List<Object?> get props => [resourceId, locationId];
}

final class TrackWebReaderProgress extends WebReaderEvent {
  const TrackWebReaderProgress(this.signal);

  final WebReaderProgressSignal signal;

  @override
  List<Object?> get props => [signal];
}

sealed class WebReaderState extends Equatable {
  const WebReaderState();

  @override
  List<Object?> get props => [];
}

final class WebReaderInitial extends WebReaderState {
  const WebReaderInitial();
}

final class WebReaderLoading extends WebReaderState {
  const WebReaderLoading();
}

final class WebReaderListLoaded extends WebReaderState {
  const WebReaderListLoaded(this.webReaders);

  final List<Resource> webReaders;

  @override
  List<Object?> get props => [webReaders];
}

final class WebReaderDetailLoaded extends WebReaderState {
  const WebReaderDetailLoaded(this.webReader);

  final WebReaderDetail webReader;

  @override
  List<Object?> get props => [webReader];
}

enum WebReaderOperationType {
  added,
  updated,
  deleted,
  locationAdded,
  locationRemoved,
  progressTracked,
}

final class WebReaderOperationSuccess extends WebReaderState {
  const WebReaderOperationSuccess(this.operationType);

  final WebReaderOperationType operationType;

  @override
  List<Object?> get props => [operationType];
}

final class WebReaderError extends WebReaderState {
  const WebReaderError(this.failure);

  final AppFailure failure;

  @override
  List<Object?> get props => [failure];
}

final class WebReaderBloc extends Bloc<WebReaderEvent, WebReaderState> {
  WebReaderBloc(this._repository) : super(const WebReaderInitial()) {
    on<LoadWebReaders>(_onLoadWebReaders);
    on<SearchWebReaders>(_onSearchWebReaders);
    on<LoadWebReaderDetail>(_onLoadWebReaderDetail);
    on<AddWebReader>(_onAddWebReader);
    on<UpdateWebReader>(_onUpdateWebReader);
    on<DeleteWebReader>(_onDeleteWebReader);
    on<AddWebReaderLocation>(_onAddWebReaderLocation);
    on<RemoveWebReaderLocation>(_onRemoveWebReaderLocation);
    on<TrackWebReaderProgress>(_onTrackWebReaderProgress);
  }

  final WebReaderRepository _repository;

  Future<void> _onLoadWebReaders(
    LoadWebReaders event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.listWebReaders();
    result.when(
      success: (webReaders) => emit(WebReaderListLoaded(webReaders)),
      failure: (failure) => emit(WebReaderError(failure)),
    );
  }

  Future<void> _onSearchWebReaders(
    SearchWebReaders event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.searchWebReaders(event.query);
    result.when(
      success: (webReaders) => emit(WebReaderListLoaded(webReaders)),
      failure: (failure) => emit(WebReaderError(failure)),
    );
  }

  Future<void> _onLoadWebReaderDetail(
    LoadWebReaderDetail event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.getWebReader(event.id);
    result.when(
      success: (webReader) => emit(WebReaderDetailLoaded(webReader)),
      failure: (failure) => emit(WebReaderError(failure)),
    );
  }

  Future<void> _onAddWebReader(
    AddWebReader event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.addWebReader(event.input);
    _emitOperationResult(result, emit, WebReaderOperationType.added);
  }

  Future<void> _onUpdateWebReader(
    UpdateWebReader event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.updateWebReader(event.id, event.input);
    _emitOperationResult(result, emit, WebReaderOperationType.updated);
  }

  Future<void> _onDeleteWebReader(
    DeleteWebReader event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.deleteWebReader(event.id);
    _emitVoidOperationResult(result, emit, WebReaderOperationType.deleted);
  }

  Future<void> _onAddWebReaderLocation(
    AddWebReaderLocation event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.addLocation(event.resourceId, event.input);
    _emitLocationOperationResult(result, emit, WebReaderOperationType.locationAdded);
  }

  Future<void> _onRemoveWebReaderLocation(
    RemoveWebReaderLocation event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.removeLocation(
      event.resourceId,
      event.locationId,
    );
    _emitVoidOperationResult(result, emit, WebReaderOperationType.locationRemoved);
  }

  Future<void> _onTrackWebReaderProgress(
    TrackWebReaderProgress event,
    Emitter<WebReaderState> emit,
  ) async {
    emit(const WebReaderLoading());
    final result = await _repository.trackProgress(event.signal);
    _emitVoidOperationResult(result, emit, WebReaderOperationType.progressTracked);
  }

  void _emitOperationResult(
    Result<Resource, AppFailure> result,
    Emitter<WebReaderState> emit,
    WebReaderOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(WebReaderOperationSuccess(operationType)),
      failure: (failure) => emit(WebReaderError(failure)),
    );
  }

  void _emitLocationOperationResult(
    Result<ResourceLocation, AppFailure> result,
    Emitter<WebReaderState> emit,
    WebReaderOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(WebReaderOperationSuccess(operationType)),
      failure: (failure) => emit(WebReaderError(failure)),
    );
  }

  void _emitVoidOperationResult(
    Result<void, AppFailure> result,
    Emitter<WebReaderState> emit,
    WebReaderOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(WebReaderOperationSuccess(operationType)),
      failure: (failure) => emit(WebReaderError(failure)),
    );
  }
}
