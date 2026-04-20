import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/batch/batch_bloc.dart';
import 'package:personal_inventory_frontend/models/batch_operations.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';

import '../../support/fake_repositories.dart';

void main() {
  group('BatchBloc', () {
    late FakeBatchOperationRepository repository;

    setUp(() {
      repository = FakeBatchOperationRepository();
    });

    test('has initial state BatchInitial', () {
      expect(BatchBloc(repository).state, const BatchInitial());
    });

    blocTest<BatchBloc, BatchState>(
      'emits loading then success when BatchImportRequested succeeds',
      build: () => BatchBloc(repository),
      act: (bloc) => bloc.add(
        const BatchImportRequested(
          BatchImportRequest(paths: ['/a', '/b'], recursive: false),
        ),
      ),
      expect: () => [
        const BatchLoading(),
        BatchSuccess(
          const BatchOperationResponse(
            type: BatchOperationType.importResources,
            results: [],
          ),
        ),
      ],
    );

    blocTest<BatchBloc, BatchState>(
      'passes import request to repository',
      build: () => BatchBloc(repository),
      act: (bloc) => bloc.add(
        const BatchImportRequested(
          BatchImportRequest(paths: ['/docs'], recursive: true),
        ),
      ),
      verify: (_) {
        expect(repository.importCalls, 1);
        expect(repository.lastImportRequest?.paths, ['/docs']);
        expect(repository.lastImportRequest?.recursive, true);
      },
    );

    blocTest<BatchBloc, BatchState>(
      'emits loading then success when BatchUpdateMetadataRequested succeeds',
      build: () => BatchBloc(
        FakeBatchOperationRepository(
          updateResult: Success(
            BatchOperationResponse(
              type: BatchOperationType.updateMetadata,
              results: [
                const BatchOperationItemResult(itemKey: 'id1', success: true),
              ],
            ),
          ),
        ),
      ),
      act: (bloc) => bloc.add(
        const BatchUpdateMetadataRequested(
          BatchMetadataUpdateRequest(
            resourceIds: ['id1'],
            fields: {'author': 'Test Author'},
          ),
        ),
      ),
      expect: () => [
        const BatchLoading(),
        BatchSuccess(
          BatchOperationResponse(
            type: BatchOperationType.updateMetadata,
            results: [
              const BatchOperationItemResult(itemKey: 'id1', success: true),
            ],
          ),
        ),
      ],
    );

    blocTest<BatchBloc, BatchState>(
      'passes update request to repository',
      build: () => BatchBloc(repository),
      act: (bloc) => bloc.add(
        const BatchUpdateMetadataRequested(
          BatchMetadataUpdateRequest(
            resourceIds: ['r1', 'r2'],
            fields: {'language': 'en'},
          ),
        ),
      ),
      verify: (_) {
        expect(repository.updateCalls, 1);
        expect(repository.lastUpdateRequest?.resourceIds, ['r1', 'r2']);
        expect(repository.lastUpdateRequest?.fields, {'language': 'en'});
      },
    );

    blocTest<BatchBloc, BatchState>(
      'emits loading then success when BatchCopyMetadataRequested succeeds',
      build: () => BatchBloc(
        FakeBatchOperationRepository(
          copyResult: Success(
            BatchOperationResponse(
              type: BatchOperationType.copyMetadata,
              results: [
                const BatchOperationItemResult(itemKey: 't1', success: true),
                const BatchOperationItemResult(itemKey: 't2', success: true),
              ],
            ),
          ),
        ),
      ),
      act: (bloc) => bloc.add(
        const BatchCopyMetadataRequested(
          BatchMetadataCopyRequest(
            sourceResourceId: 's1',
            targetResourceIds: ['t1', 't2'],
          ),
        ),
      ),
      expect: () => [
        const BatchLoading(),
        BatchSuccess(
          BatchOperationResponse(
            type: BatchOperationType.copyMetadata,
            results: [
              const BatchOperationItemResult(itemKey: 't1', success: true),
              const BatchOperationItemResult(itemKey: 't2', success: true),
            ],
          ),
        ),
      ],
    );

    blocTest<BatchBloc, BatchState>(
      'passes copy request to repository',
      build: () => BatchBloc(repository),
      act: (bloc) => bloc.add(
        const BatchCopyMetadataRequested(
          BatchMetadataCopyRequest(
            sourceResourceId: 'source',
            targetResourceIds: ['target1'],
          ),
        ),
      ),
      verify: (_) {
        expect(repository.copyCalls, 1);
        expect(repository.lastCopyRequest?.sourceResourceId, 'source');
        expect(repository.lastCopyRequest?.targetResourceIds, ['target1']);
      },
    );

    blocTest<BatchBloc, BatchState>(
      'emits BatchError when import fails',
      build: () => BatchBloc(
        FakeBatchOperationRepository(
          importResult: const Failure(NetworkFailure('connection refused')),
        ),
      ),
      act: (bloc) => bloc.add(
        const BatchImportRequested(
          BatchImportRequest(paths: ['/x'], recursive: false),
        ),
      ),
      expect: () => [
        const BatchLoading(),
        const BatchError(NetworkFailure('connection refused')),
      ],
    );

    blocTest<BatchBloc, BatchState>(
      'emits BatchError when update fails with server error',
      build: () => BatchBloc(
        FakeBatchOperationRepository(
          updateResult: const Failure(ServerFailure(500)),
        ),
      ),
      act: (bloc) => bloc.add(
        const BatchUpdateMetadataRequested(
          BatchMetadataUpdateRequest(resourceIds: ['id1'], fields: {}),
        ),
      ),
      expect: () => [
        const BatchLoading(),
        const BatchError(ServerFailure(500)),
      ],
    );
  });
}
