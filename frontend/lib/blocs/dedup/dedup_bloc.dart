import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/dedup.dart';
import '../../models/failures.dart';
import '../../models/result.dart';
import '../../repositories/dedup_repository.dart';

// --- Events ---

sealed class DedupEvent extends Equatable {
  const DedupEvent();
}

final class LoadWarnings extends DedupEvent {
  const LoadWarnings();
  @override
  List<Object?> get props => [];
}

final class ScanDuplicates extends DedupEvent {
  const ScanDuplicates();
  @override
  List<Object?> get props => [];
}

final class DismissWarning extends DedupEvent {
  const DismissWarning(this.warningId);
  final String warningId;
  @override
  List<Object?> get props => [warningId];
}

final class MergeResources extends DedupEvent {
  const MergeResources({
    required this.warningId,
    required this.keepId,
    required this.discardId,
  });
  final String warningId;
  final String keepId;
  final String discardId;
  @override
  List<Object?> get props => [warningId, keepId, discardId];
}

// --- States ---

enum DedupOperationType { dismiss, merge }

sealed class DedupState extends Equatable {
  const DedupState();
}

final class DedupInitial extends DedupState {
  const DedupInitial();
  @override
  List<Object?> get props => [];
}

final class DedupLoading extends DedupState {
  const DedupLoading();
  @override
  List<Object?> get props => [];
}

final class DedupWarningsLoaded extends DedupState {
  const DedupWarningsLoaded(this.warnings);
  final List<DedupWarning> warnings;
  @override
  List<Object?> get props => [warnings];
}

final class DedupOperationSuccess extends DedupState {
  const DedupOperationSuccess(this.operationType);
  final DedupOperationType operationType;
  @override
  List<Object?> get props => [operationType];
}

final class DedupError extends DedupState {
  const DedupError(this.failure);
  final AppFailure failure;
  @override
  List<Object?> get props => [failure];
}

// --- Bloc ---

final class DedupBloc extends Bloc<DedupEvent, DedupState> {
  DedupBloc(this._repository) : super(const DedupInitial()) {
    on<LoadWarnings>(_onLoadWarnings);
    on<ScanDuplicates>(_onScan);
    on<DismissWarning>(_onDismiss);
    on<MergeResources>(_onMerge);
  }

  final DedupRepository _repository;

  Future<void> _onLoadWarnings(
    LoadWarnings event,
    Emitter<DedupState> emit,
  ) async {
    emit(const DedupLoading());
    final result = await _repository.listPendingWarnings();
    result.when(
      success: (warnings) => emit(DedupWarningsLoaded(warnings)),
      failure: (failure) => emit(DedupError(failure)),
    );
  }

  Future<void> _onScan(
    ScanDuplicates event,
    Emitter<DedupState> emit,
  ) async {
    emit(const DedupLoading());
    final result = await _repository.scanDuplicates();
    result.when(
      success: (warnings) => emit(DedupWarningsLoaded(warnings)),
      failure: (failure) => emit(DedupError(failure)),
    );
  }

  Future<void> _onDismiss(
    DismissWarning event,
    Emitter<DedupState> emit,
  ) async {
    emit(const DedupLoading());
    final result = await _repository.dismissWarning(event.warningId);
    _emitVoidResult(result, emit, DedupOperationType.dismiss);
  }

  Future<void> _onMerge(
    MergeResources event,
    Emitter<DedupState> emit,
  ) async {
    emit(const DedupLoading());
    final result = await _repository.mergeResources(
      event.warningId,
      MergeInput(keepId: event.keepId, discardId: event.discardId),
    );
    _emitVoidResult(result, emit, DedupOperationType.merge);
  }

  void _emitVoidResult(
    Result<void, AppFailure> result,
    Emitter<DedupState> emit,
    DedupOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(DedupOperationSuccess(operationType)),
      failure: (failure) => emit(DedupError(failure)),
    );
  }
}
