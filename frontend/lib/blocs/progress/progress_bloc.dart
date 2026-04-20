import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/progress.dart';
import '../../models/resources.dart';
import '../../repositories/progress_repository.dart';

sealed class ProgressEvent extends Equatable {
  const ProgressEvent();

  @override
  List<Object?> get props => [];
}

final class LoadProgress extends ProgressEvent {
  const LoadProgress(this.resourceType, this.resourceId);

  final ResourceType resourceType;
  final String resourceId;

  @override
  List<Object?> get props => [resourceType, resourceId];
}

final class UpdateProgress extends ProgressEvent {
  const UpdateProgress(
    this.resourceType,
    this.resourceId,
    this.progress, {
    this.notes,
  });

  final ResourceType resourceType;
  final String resourceId;
  final double progress;
  final String? notes;

  @override
  List<Object?> get props => [resourceType, resourceId, progress, notes];
}

sealed class ProgressState extends Equatable {
  const ProgressState();

  @override
  List<Object?> get props => [];
}

final class ProgressInitial extends ProgressState {
  const ProgressInitial();
}

final class ProgressLoading extends ProgressState {
  const ProgressLoading();
}

final class ProgressLoaded extends ProgressState {
  const ProgressLoaded(this.progress);

  final ResourceProgress? progress;

  @override
  List<Object?> get props => [progress];
}

final class ProgressUpdated extends ProgressState {
  const ProgressUpdated(this.progress);

  final ResourceProgress progress;

  @override
  List<Object?> get props => [progress];
}

final class ProgressError extends ProgressState {
  const ProgressError(this.failure);

  final AppFailure failure;

  @override
  List<Object?> get props => [failure];
}

class ProgressBloc extends Bloc<ProgressEvent, ProgressState> {
  ProgressBloc(this._repository) : super(const ProgressInitial()) {
    on<LoadProgress>(_onLoadProgress);
    on<UpdateProgress>(_onUpdateProgress);
  }

  final ProgressRepository _repository;

  Future<void> _onLoadProgress(
    LoadProgress event,
    Emitter<ProgressState> emit,
  ) async {
    emit(const ProgressLoading());
    final result = await _repository.getProgress(
      event.resourceType,
      event.resourceId,
    );
    result.when(
      success: (progress) => emit(ProgressLoaded(progress)),
      failure: (failure) => emit(ProgressError(failure)),
    );
  }

  Future<void> _onUpdateProgress(
    UpdateProgress event,
    Emitter<ProgressState> emit,
  ) async {
    emit(const ProgressLoading());
    final result = await _repository.upsertProgress(
      event.resourceType,
      event.resourceId,
      event.progress,
      notes: event.notes,
    );
    result.when(
      success: (progress) => emit(ProgressUpdated(progress)),
      failure: (failure) => emit(ProgressError(failure)),
    );
  }
}
