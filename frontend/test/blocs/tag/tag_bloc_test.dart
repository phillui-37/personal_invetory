import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/tag/tag_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/tag.dart';
import 'package:personal_inventory_frontend/models/result.dart';

import '../../support/fake_repositories.dart';

final _createdAt = DateTime.utc(2025, 1, 1);
final _tagFantasy = Tag(id: 'tag-1', name: 'fantasy', createdAt: _createdAt);
final _tagSciFi = Tag(id: 'tag-2', name: 'sci-fi', createdAt: _createdAt);

void main() {
  group('TagBloc', () {
    blocTest<TagBloc, TagState>(
      'emits loading then tag list on LoadTags success',
      build: () => TagBloc(
        FakeTagRepository(
          listResult: Success<List<Tag>, AppFailure>([_tagFantasy, _tagSciFi]),
        ),
      ),
      act: (bloc) => bloc.add(const LoadTags()),
      expect: () => [
        const TagLoading(),
        TagListLoaded([_tagFantasy, _tagSciFi]),
      ],
    );

    blocTest<TagBloc, TagState>(
      'emits loading then error on LoadTags failure',
      build: () => TagBloc(
        FakeTagRepository(
          listResult:
              const Failure<List<Tag>, AppFailure>(NetworkFailure('offline')),
        ),
      ),
      act: (bloc) => bloc.add(const LoadTags()),
      expect: () => const [
        TagLoading(),
        TagError(NetworkFailure('offline')),
      ],
    );

    late FakeTagRepository createRepo;
    blocTest<TagBloc, TagState>(
      'emits loading then operation success on CreateTag',
      build: () {
        createRepo = FakeTagRepository(
          createResult: Success<Tag, AppFailure>(_tagFantasy),
        );
        return TagBloc(createRepo);
      },
      act: (bloc) => bloc.add(const CreateTag(' Fantasy ')),
      expect: () => const [
        TagLoading(),
        TagOperationSuccess(TagOperationType.created),
      ],
      verify: (_) {
        expect(createRepo.createCalls, 1);
        expect(createRepo.lastCreateName, ' Fantasy ');
      },
    );

    blocTest<TagBloc, TagState>(
      'emits loading then error on CreateTag failure',
      build: () => TagBloc(
        FakeTagRepository(
          createResult: const Failure<Tag, AppFailure>(ServerFailure(409)),
        ),
      ),
      act: (bloc) => bloc.add(const CreateTag('fantasy')),
      expect: () => const [
        TagLoading(),
        TagError(ServerFailure(409)),
      ],
    );

    late FakeTagRepository deleteRepo;
    blocTest<TagBloc, TagState>(
      'emits loading then operation success on DeleteTag',
      build: () {
        deleteRepo = FakeTagRepository();
        return TagBloc(deleteRepo);
      },
      act: (bloc) => bloc.add(const DeleteTag('tag-1')),
      expect: () => const [
        TagLoading(),
        TagOperationSuccess(TagOperationType.deleted),
      ],
      verify: (_) {
        expect(deleteRepo.deleteCalls, 1);
        expect(deleteRepo.lastDeleteId, 'tag-1');
      },
    );

    late FakeTagRepository resourceTagsRepo;
    blocTest<TagBloc, TagState>(
      'emits loading then resource tags on LoadResourceTags success',
      build: () {
        resourceTagsRepo = FakeTagRepository(
          tagsForResourceResult:
              Success<List<Tag>, AppFailure>([_tagFantasy, _tagSciFi]),
        );
        return TagBloc(resourceTagsRepo);
      },
      act: (bloc) => bloc.add(
        const LoadResourceTags(ResourceType.webReader, 'resource-1'),
      ),
      expect: () => [
        const TagLoading(),
        ResourceTagsLoaded([_tagFantasy, _tagSciFi]),
      ],
      verify: (_) {
        expect(resourceTagsRepo.tagsForResourceCalls, 1);
        expect(
            resourceTagsRepo.lastTagsForResourceType, ResourceType.webReader);
        expect(resourceTagsRepo.lastTagsForResourceId, 'resource-1');
      },
    );

    late FakeTagRepository attachRepo;
    blocTest<TagBloc, TagState>(
      'emits loading then operation success on AttachTag',
      build: () {
        attachRepo = FakeTagRepository();
        return TagBloc(attachRepo);
      },
      act: (bloc) => bloc.add(
        const AttachTag(ResourceType.ebook, 'resource-1', 'tag-2'),
      ),
      expect: () => const [
        TagLoading(),
        TagOperationSuccess(TagOperationType.attached),
      ],
      verify: (_) {
        expect(attachRepo.attachCalls, 1);
        expect(attachRepo.lastAttachType, ResourceType.ebook);
        expect(attachRepo.lastAttachResourceId, 'resource-1');
        expect(attachRepo.lastAttachTagId, 'tag-2');
      },
    );

    late FakeTagRepository detachRepo;
    blocTest<TagBloc, TagState>(
      'emits loading then operation success on DetachTag',
      build: () {
        detachRepo = FakeTagRepository();
        return TagBloc(detachRepo);
      },
      act: (bloc) => bloc.add(
        const DetachTag(ResourceType.ebook, 'resource-1', 'tag-2'),
      ),
      expect: () => const [
        TagLoading(),
        TagOperationSuccess(TagOperationType.detached),
      ],
      verify: (_) {
        expect(detachRepo.detachCalls, 1);
        expect(detachRepo.lastDetachType, ResourceType.ebook);
        expect(detachRepo.lastDetachResourceId, 'resource-1');
        expect(detachRepo.lastDetachTagId, 'tag-2');
      },
    );
  });
}
