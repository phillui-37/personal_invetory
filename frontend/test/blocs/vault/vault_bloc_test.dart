import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/vault/vault_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/vault.dart';
import 'package:personal_inventory_frontend/repositories/vault_repository.dart';

void main() {
  group('VaultBloc', () {
    blocTest<VaultBloc, VaultState>(
      'emits loading then status loaded on CheckVaultStatus success',
      build: () => VaultBloc(_FakeVaultRepo(
        onGetStatus: () async =>
            const Success(VaultStatus(initialized: true, unlocked: false)),
      )),
      act: (bloc) => bloc.add(const CheckVaultStatus()),
      expect: () => const [
        VaultLoading(),
        VaultStatusLoaded(VaultStatus(initialized: true, unlocked: false)),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then error on CheckVaultStatus failure',
      build: () => VaultBloc(_FakeVaultRepo(
        onGetStatus: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const CheckVaultStatus()),
      expect: () => const [
        VaultLoading(),
        VaultError(NetworkFailure('offline')),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then operation success on InitializeVault',
      build: () => VaultBloc(_FakeVaultRepo(
        onInitialize: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const InitializeVault('mypassword')),
      expect: () => const [
        VaultLoading(),
        VaultOperationSuccess(VaultOperationType.initialize),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then error on InitializeVault failure',
      build: () => VaultBloc(_FakeVaultRepo(
        onInitialize: (_) async => const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const InitializeVault('mypassword')),
      expect: () => const [
        VaultLoading(),
        VaultError(ServerFailure(500)),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then operation success on UnlockVault',
      build: () => VaultBloc(_FakeVaultRepo(
        onUnlock: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const UnlockVault('mypassword')),
      expect: () => const [
        VaultLoading(),
        VaultOperationSuccess(VaultOperationType.unlock),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then error on UnlockVault failure',
      build: () => VaultBloc(_FakeVaultRepo(
        onUnlock: (_) async => const Failure(ServerFailure(401)),
      )),
      act: (bloc) => bloc.add(const UnlockVault('wrong')),
      expect: () => const [
        VaultLoading(),
        VaultError(ServerFailure(401)),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then operation success on LockVault',
      build: () => VaultBloc(_FakeVaultRepo(
        onLock: () async => const Success(null),
      )),
      act: (bloc) => bloc.add(const LockVault()),
      expect: () => const [
        VaultLoading(),
        VaultOperationSuccess(VaultOperationType.lock),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then platforms loaded on LoadPlatforms',
      build: () => VaultBloc(_FakeVaultRepo(
        onListPlatforms: () async => const Success(['steam', 'dlsite']),
      )),
      act: (bloc) => bloc.add(const LoadPlatforms()),
      expect: () => const [
        VaultLoading(),
        VaultPlatformsLoaded(['steam', 'dlsite']),
      ],
    );
  });
}

final class _FakeVaultRepo implements VaultRepository {
  _FakeVaultRepo({
    this.onGetStatus,
    this.onInitialize,
    this.onUnlock,
    this.onLock,
    this.onStoreCredential,
    this.onRetrieveCredential,
    this.onDeleteCredential,
    this.onListPlatforms,
  });

  final Future<Result<VaultStatus, AppFailure>> Function()? onGetStatus;
  final Future<Result<void, AppFailure>> Function(String)? onInitialize;
  final Future<Result<void, AppFailure>> Function(String)? onUnlock;
  final Future<Result<void, AppFailure>> Function()? onLock;
  final Future<Result<void, AppFailure>> Function(StoreCredentialInput)?
      onStoreCredential;
  final Future<Result<String, AppFailure>> Function(RetrieveCredentialInput)?
      onRetrieveCredential;
  final Future<Result<void, AppFailure>> Function(RetrieveCredentialInput)?
      onDeleteCredential;
  final Future<Result<List<String>, AppFailure>> Function()? onListPlatforms;

  @override
  Future<Result<VaultStatus, AppFailure>> getStatus() =>
      onGetStatus?.call() ??
      Future.value(const Failure(ServerFailure(500)));
  @override
  Future<Result<void, AppFailure>> initialize(String p) =>
      onInitialize?.call(p) ??
      Future.value(const Failure(ServerFailure(500)));
  @override
  Future<Result<void, AppFailure>> unlock(String p) =>
      onUnlock?.call(p) ??
      Future.value(const Failure(ServerFailure(500)));
  @override
  Future<Result<void, AppFailure>> lock() =>
      onLock?.call() ??
      Future.value(const Failure(ServerFailure(500)));
  @override
  Future<Result<void, AppFailure>> storeCredential(StoreCredentialInput i) =>
      onStoreCredential?.call(i) ??
      Future.value(const Failure(ServerFailure(500)));
  @override
  Future<Result<String, AppFailure>> retrieveCredential(
          RetrieveCredentialInput i) =>
      onRetrieveCredential?.call(i) ??
      Future.value(const Failure(ServerFailure(500)));
  @override
  Future<Result<void, AppFailure>> deleteCredential(
          RetrieveCredentialInput i) =>
      onDeleteCredential?.call(i) ??
      Future.value(const Failure(ServerFailure(500)));
  @override
  Future<Result<List<String>, AppFailure>> listPlatforms() =>
      onListPlatforms?.call() ??
      Future.value(const Failure(ServerFailure(500)));
}
