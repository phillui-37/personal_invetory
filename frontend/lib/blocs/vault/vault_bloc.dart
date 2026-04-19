import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/result.dart';
import '../../models/vault.dart';
import '../../repositories/vault_repository.dart';

// --- Events ---

sealed class VaultEvent extends Equatable {
  const VaultEvent();
}

final class CheckVaultStatus extends VaultEvent {
  const CheckVaultStatus();
  @override
  List<Object?> get props => [];
}

final class InitializeVault extends VaultEvent {
  const InitializeVault(this.masterPassword);
  final String masterPassword;
  @override
  List<Object?> get props => [];
}

final class UnlockVault extends VaultEvent {
  const UnlockVault(this.masterPassword);
  final String masterPassword;
  @override
  List<Object?> get props => [];
}

final class LockVault extends VaultEvent {
  const LockVault();
  @override
  List<Object?> get props => [];
}

final class LoadPlatforms extends VaultEvent {
  const LoadPlatforms();
  @override
  List<Object?> get props => [];
}

final class StoreCredential extends VaultEvent {
  const StoreCredential({
    required this.platform,
    required this.credentialType,
    required this.plaintext,
  });
  final String platform;
  final String credentialType;
  final String plaintext;
  @override
  List<Object?> get props => [platform, credentialType];
}

// --- States ---

enum VaultOperationType { initialize, unlock, lock, storeCredential }

sealed class VaultState extends Equatable {
  const VaultState();
}

final class VaultInitial extends VaultState {
  const VaultInitial();
  @override
  List<Object?> get props => [];
}

final class VaultLoading extends VaultState {
  const VaultLoading();
  @override
  List<Object?> get props => [];
}

final class VaultStatusLoaded extends VaultState {
  const VaultStatusLoaded(this.status);
  final VaultStatus status;
  @override
  List<Object?> get props => [status];
}

final class VaultPlatformsLoaded extends VaultState {
  const VaultPlatformsLoaded(this.platforms);
  final List<String> platforms;
  @override
  List<Object?> get props => [platforms];
}

final class VaultOperationSuccess extends VaultState {
  const VaultOperationSuccess(this.operationType);
  final VaultOperationType operationType;
  @override
  List<Object?> get props => [operationType];
}

final class VaultError extends VaultState {
  const VaultError(this.failure);
  final AppFailure failure;
  @override
  List<Object?> get props => [failure];
}

// --- Bloc ---

final class VaultBloc extends Bloc<VaultEvent, VaultState> {
  VaultBloc(this._repository) : super(const VaultInitial()) {
    on<CheckVaultStatus>(_onCheckStatus);
    on<InitializeVault>(_onInitialize);
    on<UnlockVault>(_onUnlock);
    on<LockVault>(_onLock);
    on<LoadPlatforms>(_onLoadPlatforms);
    on<StoreCredential>(_onStoreCredential);
  }

  final VaultRepository _repository;

  Future<void> _onCheckStatus(
    CheckVaultStatus event,
    Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.getStatus();
    result.when(
      success: (status) => emit(VaultStatusLoaded(status)),
      failure: (failure) => emit(VaultError(failure)),
    );
  }

  Future<void> _onInitialize(
    InitializeVault event,
    Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.initialize(event.masterPassword);
    _emitVoidResult(result, emit, VaultOperationType.initialize);
  }

  Future<void> _onUnlock(
    UnlockVault event,
    Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.unlock(event.masterPassword);
    _emitVoidResult(result, emit, VaultOperationType.unlock);
  }

  Future<void> _onLock(
    LockVault event,
    Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.lock();
    _emitVoidResult(result, emit, VaultOperationType.lock);
  }

  Future<void> _onLoadPlatforms(
    LoadPlatforms event,
    Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.listPlatforms();
    result.when(
      success: (platforms) => emit(VaultPlatformsLoaded(platforms)),
      failure: (failure) => emit(VaultError(failure)),
    );
  }

  Future<void> _onStoreCredential(
    StoreCredential event,
    Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.storeCredential(
      StoreCredentialInput(
        platform: event.platform,
        credentialType: event.credentialType,
        plaintext: event.plaintext,
      ),
    );
    _emitVoidResult(result, emit, VaultOperationType.storeCredential);
  }

  void _emitVoidResult(
    Result<void, AppFailure> result,
    Emitter<VaultState> emit,
    VaultOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(VaultOperationSuccess(operationType)),
      failure: (failure) => emit(VaultError(failure)),
    );
  }
}
