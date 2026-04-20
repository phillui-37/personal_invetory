import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/batch_operations.dart';
import '../../models/failures.dart';
import '../../repositories/batch_operation_repository.dart';

// ── Events ──────────────────────────────────────────────────────────────────

sealed class BatchEvent extends Equatable {
  const BatchEvent();

  @override
  List<Object?> get props => [];
}

final class BatchImportRequested extends BatchEvent {
  const BatchImportRequested(this.request);

  final BatchImportRequest request;

  @override
  List<Object?> get props => [request];
}

final class BatchUpdateMetadataRequested extends BatchEvent {
  const BatchUpdateMetadataRequested(this.request);

  final BatchMetadataUpdateRequest request;

  @override
  List<Object?> get props => [request];
}

final class BatchCopyMetadataRequested extends BatchEvent {
  const BatchCopyMetadataRequested(this.request);

  final BatchMetadataCopyRequest request;

  @override
  List<Object?> get props => [request];
}

// ── States ───────────────────────────────────────────────────────────────────

sealed class BatchState extends Equatable {
  const BatchState();

  @override
  List<Object?> get props => [];
}

final class BatchInitial extends BatchState {
  const BatchInitial();
}

final class BatchLoading extends BatchState {
  const BatchLoading();
}

final class BatchSuccess extends BatchState {
  const BatchSuccess(this.response);

  final BatchOperationResponse response;

  @override
  List<Object?> get props => [response];
}

final class BatchError extends BatchState {
  const BatchError(this.failure);

  final AppFailure failure;

  @override
  List<Object?> get props => [failure];
}

// ── BLoC ─────────────────────────────────────────────────────────────────────

class BatchBloc extends Bloc<BatchEvent, BatchState> {
  BatchBloc(this._repository) : super(const BatchInitial()) {
    on<BatchImportRequested>(_onImport);
    on<BatchUpdateMetadataRequested>(_onUpdateMetadata);
    on<BatchCopyMetadataRequested>(_onCopyMetadata);
  }

  final BatchOperationRepository _repository;

  Future<void> _onImport(
    BatchImportRequested event,
    Emitter<BatchState> emit,
  ) async {
    emit(const BatchLoading());
    final result = await _repository.batchImport(event.request);
    result.when(
      success: (response) => emit(BatchSuccess(response)),
      failure: (failure) => emit(BatchError(failure)),
    );
  }

  Future<void> _onUpdateMetadata(
    BatchUpdateMetadataRequested event,
    Emitter<BatchState> emit,
  ) async {
    emit(const BatchLoading());
    final result = await _repository.batchUpdateMetadata(event.request);
    result.when(
      success: (response) => emit(BatchSuccess(response)),
      failure: (failure) => emit(BatchError(failure)),
    );
  }

  Future<void> _onCopyMetadata(
    BatchCopyMetadataRequested event,
    Emitter<BatchState> emit,
  ) async {
    emit(const BatchLoading());
    final result = await _repository.batchCopyMetadata(event.request);
    result.when(
      success: (response) => emit(BatchSuccess(response)),
      failure: (failure) => emit(BatchError(failure)),
    );
  }
}
