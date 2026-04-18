import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/dedup/dedup_bloc.dart';
import 'package:personal_inventory_frontend/models/dedup.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/dedup_repository.dart';

void main() {
  const warning1 = DedupWarning(
    id: 'w1',
    resourceIdA: 'a1',
    resourceIdB: 'b1',
    similarityScore: 0.92,
    status: DedupWarningStatus.pending,
  );
  const warning2 = DedupWarning(
    id: 'w2',
    resourceIdA: 'a2',
    resourceIdB: 'b2',
    similarityScore: 0.88,
    status: DedupWarningStatus.pending,
  );

  group('DedupBloc', () {
    blocTest<DedupBloc, DedupState>(
      'emits loading then warnings loaded on LoadWarnings success',
      build: () => DedupBloc(_FakeDedupRepo(
        onListPending: () async => const Success([warning1, warning2]),
      )),
      act: (bloc) => bloc.add(const LoadWarnings()),
      expect: () => const [
        DedupLoading(),
        DedupWarningsLoaded([warning1, warning2]),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then error on LoadWarnings failure',
      build: () => DedupBloc(_FakeDedupRepo(
        onListPending: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const LoadWarnings()),
      expect: () => const [
        DedupLoading(),
        DedupError(NetworkFailure('offline')),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then warnings loaded on ScanDuplicates success',
      build: () => DedupBloc(_FakeDedupRepo(
        onScan: () async => const Success([warning1]),
      )),
      act: (bloc) => bloc.add(const ScanDuplicates()),
      expect: () => const [
        DedupLoading(),
        DedupWarningsLoaded([warning1]),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then operation success on DismissWarning',
      build: () => DedupBloc(_FakeDedupRepo(
        onDismiss: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const DismissWarning('w1')),
      expect: () => const [
        DedupLoading(),
        DedupOperationSuccess(DedupOperationType.dismiss),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then error on DismissWarning failure',
      build: () => DedupBloc(_FakeDedupRepo(
        onDismiss: (_) async => const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const DismissWarning('w1')),
      expect: () => const [
        DedupLoading(),
        DedupError(ServerFailure(500)),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then operation success on MergeResources',
      build: () => DedupBloc(_FakeDedupRepo(
        onMerge: (_, __) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const MergeResources(
        warningId: 'w1',
        keepId: 'a1',
        discardId: 'b1',
      )),
      expect: () => const [
        DedupLoading(),
        DedupOperationSuccess(DedupOperationType.merge),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then error on MergeResources failure',
      build: () => DedupBloc(_FakeDedupRepo(
        onMerge: (_, __) async => const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const MergeResources(
        warningId: 'w1',
        keepId: 'a1',
        discardId: 'b1',
      )),
      expect: () => const [
        DedupLoading(),
        DedupError(ServerFailure(500)),
      ],
    );
  });
}

final class _FakeDedupRepo implements DedupRepository {
  _FakeDedupRepo({this.onScan, this.onListPending, this.onDismiss, this.onMerge});

  final Future<Result<List<DedupWarning>, AppFailure>> Function()? onScan;
  final Future<Result<List<DedupWarning>, AppFailure>> Function()? onListPending;
  final Future<Result<void, AppFailure>> Function(String)? onDismiss;
  final Future<Result<void, AppFailure>> Function(String, MergeInput)? onMerge;

  @override
  Future<Result<List<DedupWarning>, AppFailure>> scanDuplicates() =>
      onScan?.call() ?? Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<List<DedupWarning>, AppFailure>> listPendingWarnings() =>
      onListPending?.call() ?? Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<void, AppFailure>> dismissWarning(String id) =>
      onDismiss?.call(id) ?? Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<void, AppFailure>> mergeResources(String warningId, MergeInput input) =>
      onMerge?.call(warningId, input) ?? Future.value(const Failure(ServerFailure(500)));
}
