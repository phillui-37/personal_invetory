import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/progress/progress_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/progress.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';

import '../../support/fake_repositories.dart';

void main() {
  group('ProgressBloc', () {
    late FakeProgressRepository repository;

    setUp(() {
      repository = FakeProgressRepository();
    });

    test('has initial state ProgressInitial', () {
      expect(
        ProgressBloc(FakeProgressRepository()).state,
        const ProgressInitial(),
      );
    });

    blocTest<ProgressBloc, ProgressState>(
      'emits loading then loaded when LoadProgress succeeds',
      build: () => ProgressBloc(
        FakeProgressRepository(
          getResult: Success<ResourceProgress?, AppFailure>(
            ResourceProgress(
              resourceId: 'w1',
              progress: 0.42,
              notes: 'Chapter 2',
              updatedAt: DateTime.utc(2025, 1, 1),
            ),
          ),
        ),
      ),
      act: (bloc) => bloc.add(
        const LoadProgress(ResourceType.webReader, 'w1'),
      ),
      expect: () => [
        const ProgressLoading(),
        ProgressLoaded(
          ResourceProgress(
            resourceId: 'w1',
            progress: 0.42,
            notes: 'Chapter 2',
            updatedAt: DateTime.utc(2025, 1, 1),
          ),
        ),
      ],
    );

    blocTest<ProgressBloc, ProgressState>(
      'passes resource type and id to repository when loading progress',
      build: () => ProgressBloc(repository),
      act: (bloc) => bloc.add(
        const LoadProgress(ResourceType.video, 'video-1'),
      ),
      verify: (_) {
        expect(repository.getCalls, 1);
        expect(repository.lastGetResourceType, ResourceType.video);
        expect(repository.lastGetResourceId, 'video-1');
      },
    );

    blocTest<ProgressBloc, ProgressState>(
      'emits loading then updated when UpdateProgress succeeds',
      build: () => ProgressBloc(
        FakeProgressRepository(
          upsertResult: Success<ResourceProgress, AppFailure>(
            ResourceProgress(
              resourceId: 'w1',
              progress: 0.75,
              notes: 'Chapter 5',
              updatedAt: DateTime.utc(2025, 1, 2),
            ),
          ),
        ),
      ),
      act: (bloc) => bloc.add(
        const UpdateProgress(
          ResourceType.webReader,
          'w1',
          0.75,
          notes: 'Chapter 5',
        ),
      ),
      expect: () => [
        const ProgressLoading(),
        ProgressUpdated(
          ResourceProgress(
            resourceId: 'w1',
            progress: 0.75,
            notes: 'Chapter 5',
            updatedAt: DateTime.utc(2025, 1, 2),
          ),
        ),
      ],
    );

    blocTest<ProgressBloc, ProgressState>(
      'emits error when UpdateProgress fails',
      build: () => ProgressBloc(
        FakeProgressRepository(
          upsertResult: const Failure<ResourceProgress, AppFailure>(
            ServerFailure(500),
          ),
        ),
      ),
      act: (bloc) => bloc.add(
        const UpdateProgress(ResourceType.ebook, 'e1', 0.1),
      ),
      expect: () => const [
        ProgressLoading(),
        ProgressError(ServerFailure(500)),
      ],
    );
  });
}
