import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/sync.dart';
import '../../repositories/sync_repository.dart';

// --- Events ---

sealed class SyncEvent extends Equatable {
  const SyncEvent();
}

final class LoadEcosystemStatus extends SyncEvent {
  const LoadEcosystemStatus();
  @override
  List<Object?> get props => [];
}

final class LoadPlatformSyncs extends SyncEvent {
  const LoadPlatformSyncs(this.platform);
  final String platform;
  @override
  List<Object?> get props => [platform];
}

final class TriggerSync extends SyncEvent {
  const TriggerSync(this.platform);
  final String platform;
  @override
  List<Object?> get props => [platform];
}

// --- States ---

sealed class SyncState extends Equatable {
  const SyncState();
}

final class SyncInitial extends SyncState {
  const SyncInitial();
  @override
  List<Object?> get props => [];
}

final class SyncLoading extends SyncState {
  const SyncLoading();
  @override
  List<Object?> get props => [];
}

final class EcosystemStatusLoaded extends SyncState {
  const EcosystemStatusLoaded(this.platforms);
  final List<PlatformStatus> platforms;
  @override
  List<Object?> get props => [platforms];
}

final class PlatformSyncsLoaded extends SyncState {
  const PlatformSyncsLoaded({required this.platform, required this.jobs});
  final String platform;
  final List<SyncJob> jobs;
  @override
  List<Object?> get props => [platform, jobs];
}

final class SyncTriggered extends SyncState {
  const SyncTriggered(this.job);
  final SyncJob job;
  @override
  List<Object?> get props => [job];
}

final class SyncError extends SyncState {
  const SyncError(this.failure);
  final AppFailure failure;
  @override
  List<Object?> get props => [failure];
}

// --- Bloc ---

class SyncBloc extends Bloc<SyncEvent, SyncState> {
  SyncBloc(this._repo) : super(const SyncInitial()) {
    on<LoadEcosystemStatus>(_onLoadEcosystemStatus);
    on<LoadPlatformSyncs>(_onLoadPlatformSyncs);
    on<TriggerSync>(_onTriggerSync);
  }

  final SyncRepository _repo;

  Future<void> _onLoadEcosystemStatus(
    LoadEcosystemStatus event,
    Emitter<SyncState> emit,
  ) async {
    emit(const SyncLoading());
    final result = await _repo.getEcosystemStatus();
    result.when(
      success: (platforms) => emit(EcosystemStatusLoaded(platforms)),
      failure: (f) => emit(SyncError(f)),
    );
  }

  Future<void> _onLoadPlatformSyncs(
    LoadPlatformSyncs event,
    Emitter<SyncState> emit,
  ) async {
    emit(const SyncLoading());
    final result = await _repo.listPlatformSyncs(event.platform);
    result.when(
      success: (jobs) =>
          emit(PlatformSyncsLoaded(platform: event.platform, jobs: jobs)),
      failure: (f) => emit(SyncError(f)),
    );
  }

  Future<void> _onTriggerSync(
    TriggerSync event,
    Emitter<SyncState> emit,
  ) async {
    emit(const SyncLoading());
    final result = await _repo.triggerSync(event.platform);
    result.when(
      success: (job) => emit(SyncTriggered(job)),
      failure: (f) => emit(SyncError(f)),
    );
  }
}
