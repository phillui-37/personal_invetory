import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/progress.dart';
import 'package:personal_inventory_frontend/models/repository_inputs.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/tag.dart';
import 'package:personal_inventory_frontend/repositories/image_repository.dart';
import 'package:personal_inventory_frontend/repositories/in_memory_repositories.dart';
import 'package:personal_inventory_frontend/repositories/video_repository.dart';
import 'package:personal_inventory_frontend/repositories/game_repository.dart';

void main() {
  group('InMemoryProgressRepository', () {
    late InMemoryProgressRepository repo;

    setUp(() {
      repo = InMemoryProgressRepository();
    });

    test('getProgress returns null when no record exists', () async {
      final result = await repo.getProgress(ResourceType.ebook, 'res-1');

      expect(result, isA<Success<ResourceProgress?, AppFailure>>());
      expect((result as Success<ResourceProgress?, AppFailure>).value, isNull);
    });

    test('upsertProgress stores and returns the record', () async {
      final result = await repo.upsertProgress(
        ResourceType.ebook,
        'res-1',
        0.4,
        notes: 'chapter 5',
      );

      expect(result, isA<Success<ResourceProgress, AppFailure>>());
      final progress = (result as Success<ResourceProgress, AppFailure>).value;
      expect(progress.resourceId, 'res-1');
      expect(progress.progress, 0.4);
      expect(progress.notes, 'chapter 5');

      final stored = await repo.getProgress(ResourceType.ebook, 'res-1');
      expect(
        (stored as Success<ResourceProgress?, AppFailure>).value,
        equals(progress),
      );
    });
  });

  group('InMemoryImageRepository', () {
    late InMemoryImageRepository repo;

    setUp(() {
      repo = InMemoryImageRepository();
    });

    const resource = Resource(id: 'img-1', title: 'Photo A', resourceType: ResourceType.image);
    const meta = ImageMeta(resourceId: 'img-1', width: 1920, height: 1080, fileFormat: 'png', fileSizeBytes: 500000);

    test('addImage stores and returns the resource', () async {
      final result = await repo.addImage(NewImageInput(resource: resource, meta: meta));
      expect(result, isA<Success<Resource, AppFailure>>());
      expect((result as Success).value, equals(resource));
    });

    test('getImage returns detail for existing id', () async {
      await repo.addImage(NewImageInput(resource: resource, meta: meta));
      final result = await repo.getImage('img-1');
      expect(result, isA<Success<ImageDetail, AppFailure>>());
      final detail = (result as Success<ImageDetail, AppFailure>).value;
      expect(detail.resource, equals(resource));
      expect(detail.meta, equals(meta));
      expect(detail.locations, isEmpty);
    });

    test('getImage returns NotFoundFailure for missing id', () async {
      final result = await repo.getImage('no-such-id');
      expect(result, isA<Failure<ImageDetail, AppFailure>>());
      expect((result as Failure).failure, isA<NotFoundFailure>());
    });

    test('listImages returns all stored resources', () async {
      const r2 = Resource(id: 'img-2', title: 'Photo B', resourceType: ResourceType.image);
      const m2 = ImageMeta(resourceId: 'img-2');
      await repo.addImage(NewImageInput(resource: resource, meta: meta));
      await repo.addImage(NewImageInput(resource: r2, meta: m2));

      final result = await repo.listImages();
      expect((result as Success).value, hasLength(2));
    });

    test('searchImages filters by title', () async {
      const r2 = Resource(id: 'img-2', title: 'Landscape', resourceType: ResourceType.image);
      const m2 = ImageMeta(resourceId: 'img-2');
      await repo.addImage(NewImageInput(resource: resource, meta: meta));
      await repo.addImage(NewImageInput(resource: r2, meta: m2));

      final result = await repo.searchImages('photo');
      final list = (result as Success<List<Resource>, AppFailure>).value;
      expect(list, hasLength(1));
      expect(list.first.title, 'Photo A');
    });

    test('updateImage updates title and meta fields', () async {
      await repo.addImage(NewImageInput(resource: resource, meta: meta));
      final result = await repo.updateImage(
        'img-1',
        const UpdateImageInput(title: 'Photo A Updated', width: 3840),
      );
      expect(result, isA<Success<Resource, AppFailure>>());
      expect((result as Success<Resource, AppFailure>).value.title, 'Photo A Updated');

      final detail = (await repo.getImage('img-1') as Success<ImageDetail, AppFailure>).value;
      expect(detail.meta.width, 3840);
      expect(detail.meta.height, 1080); // unchanged
    });

    test('updateImage returns NotFoundFailure for missing id', () async {
      final result = await repo.updateImage('no-id', const UpdateImageInput(title: 'x'));
      expect(result, isA<Failure<Resource, AppFailure>>());
    });

    test('deleteImage removes the resource', () async {
      await repo.addImage(NewImageInput(resource: resource, meta: meta));
      await repo.deleteImage('img-1');
      final result = await repo.getImage('img-1');
      expect(result, isA<Failure<ImageDetail, AppFailure>>());
    });

    test('addLocation attaches a location to a resource', () async {
      await repo.addImage(NewImageInput(resource: resource, meta: meta));
      final result = await repo.addLocation(
        'img-1',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/pics/a.png', storageType: StorageType.localFs),
      );
      expect(result, isA<Success<ResourceLocation, AppFailure>>());

      final detail = (await repo.getImage('img-1') as Success<ImageDetail, AppFailure>).value;
      expect(detail.locations, hasLength(1));
    });

    test('addLocation returns NotFoundFailure for missing resource', () async {
      final result = await repo.addLocation(
        'no-id',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/x', storageType: StorageType.localFs),
      );
      expect(result, isA<Failure<ResourceLocation, AppFailure>>());
    });

    test('removeLocation removes a location from a resource', () async {
      await repo.addImage(NewImageInput(resource: resource, meta: meta));
      final locResult = await repo.addLocation(
        'img-1',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/pics/a.png', storageType: StorageType.localFs),
      );
      final locId = (locResult as Success<ResourceLocation, AppFailure>).value.id;
      await repo.removeLocation('img-1', locId);

      final detail = (await repo.getImage('img-1') as Success<ImageDetail, AppFailure>).value;
      expect(detail.locations, isEmpty);
    });

    test('removeLocation returns NotFoundFailure for missing resource', () async {
      final result = await repo.removeLocation('no-id', 'loc-1');
      expect(result, isA<Failure<void, AppFailure>>());
    });
  });

  group('InMemoryTagRepository', () {
    late InMemoryTagRepository repo;

    setUp(() {
      repo = InMemoryTagRepository();
    });

    test('listTags returns empty initially', () async {
      final result = await repo.listTags();

      expect(result, isA<Success<List<Tag>, AppFailure>>());
      expect((result as Success<List<Tag>, AppFailure>).value, isEmpty);
    });

    test('createTag stores normalized tag and listTags returns it', () async {
      final createResult = await repo.createTag('  Sci-Fi  ');

      expect(createResult, isA<Success<Tag, AppFailure>>());
      final created = (createResult as Success<Tag, AppFailure>).value;
      expect(created.name, 'sci-fi');

      final listResult = await repo.listTags();
      final tags = (listResult as Success<List<Tag>, AppFailure>).value;
      expect(tags, contains(created));
    });

    test('tagsForResource returns tags attached to a resource', () async {
      final created = (await repo.createTag('fantasy') as Success<Tag, AppFailure>).value;
      await repo.attachTag(ResourceType.ebook, 'res-1', created.id);

      final tagsResult = await repo.tagsForResource(ResourceType.ebook, 'res-1');
      final tags = (tagsResult as Success<List<Tag>, AppFailure>).value;

      expect(tags, equals([created]));
    });

    test('detachTag removes resource association', () async {
      final created = (await repo.createTag('mystery') as Success<Tag, AppFailure>).value;
      await repo.attachTag(ResourceType.ebook, 'res-1', created.id);

      await repo.detachTag(ResourceType.ebook, 'res-1', created.id);

      final tagsResult = await repo.tagsForResource(ResourceType.ebook, 'res-1');
      expect((tagsResult as Success<List<Tag>, AppFailure>).value, isEmpty);
    });

    test('deleteTag removes tag and cascades resource associations', () async {
      final created = (await repo.createTag('history') as Success<Tag, AppFailure>).value;
      await repo.attachTag(ResourceType.ebook, 'res-1', created.id);

      await repo.deleteTag(created.id);

      final tagsResult = await repo.tagsForResource(ResourceType.ebook, 'res-1');
      expect((tagsResult as Success<List<Tag>, AppFailure>).value, isEmpty);
      final listResult = await repo.listTags();
      expect((listResult as Success<List<Tag>, AppFailure>).value, isEmpty);
    });
  });

  group('InMemoryVideoRepository', () {
    late InMemoryVideoRepository repo;

    setUp(() {
      repo = InMemoryVideoRepository();
    });

    const resource = Resource(id: 'vid-1', title: 'Clip A', resourceType: ResourceType.video);
    const meta = VideoMeta(resourceId: 'vid-1', durationSecs: 3600, fileFormat: 'mp4', resolution: '1080p', fileSizeBytes: 2000000);

    test('addVideo stores and returns the resource', () async {
      final result = await repo.addVideo(NewVideoInput(resource: resource, meta: meta));
      expect(result, isA<Success<Resource, AppFailure>>());
      expect((result as Success).value, equals(resource));
    });

    test('getVideo returns detail for existing id', () async {
      await repo.addVideo(NewVideoInput(resource: resource, meta: meta));
      final result = await repo.getVideo('vid-1');
      expect(result, isA<Success<VideoDetail, AppFailure>>());
      final detail = (result as Success<VideoDetail, AppFailure>).value;
      expect(detail.resource, equals(resource));
      expect(detail.meta, equals(meta));
      expect(detail.locations, isEmpty);
    });

    test('getVideo returns NotFoundFailure for missing id', () async {
      final result = await repo.getVideo('no-such-id');
      expect(result, isA<Failure<VideoDetail, AppFailure>>());
      expect((result as Failure).failure, isA<NotFoundFailure>());
    });

    test('listVideos returns all stored resources', () async {
      const r2 = Resource(id: 'vid-2', title: 'Clip B', resourceType: ResourceType.video);
      const m2 = VideoMeta(resourceId: 'vid-2');
      await repo.addVideo(NewVideoInput(resource: resource, meta: meta));
      await repo.addVideo(NewVideoInput(resource: r2, meta: m2));

      final result = await repo.listVideos();
      expect((result as Success).value, hasLength(2));
    });

    test('searchVideos filters by title', () async {
      const r2 = Resource(id: 'vid-2', title: 'Documentary', resourceType: ResourceType.video);
      const m2 = VideoMeta(resourceId: 'vid-2');
      await repo.addVideo(NewVideoInput(resource: resource, meta: meta));
      await repo.addVideo(NewVideoInput(resource: r2, meta: m2));

      final result = await repo.searchVideos('clip');
      final list = (result as Success<List<Resource>, AppFailure>).value;
      expect(list, hasLength(1));
      expect(list.first.title, 'Clip A');
    });

    test('updateVideo updates title and meta fields', () async {
      await repo.addVideo(NewVideoInput(resource: resource, meta: meta));
      final result = await repo.updateVideo(
        'vid-1',
        const UpdateVideoInput(title: 'Clip A Updated', resolution: '4K'),
      );
      expect(result, isA<Success<Resource, AppFailure>>());
      expect((result as Success<Resource, AppFailure>).value.title, 'Clip A Updated');

      final detail = (await repo.getVideo('vid-1') as Success<VideoDetail, AppFailure>).value;
      expect(detail.meta.resolution, '4K');
      expect(detail.meta.durationSecs, 3600); // unchanged
    });

    test('updateVideo returns NotFoundFailure for missing id', () async {
      final result = await repo.updateVideo('no-id', const UpdateVideoInput(title: 'x'));
      expect(result, isA<Failure<Resource, AppFailure>>());
    });

    test('deleteVideo removes the resource', () async {
      await repo.addVideo(NewVideoInput(resource: resource, meta: meta));
      await repo.deleteVideo('vid-1');
      final result = await repo.getVideo('vid-1');
      expect(result, isA<Failure<VideoDetail, AppFailure>>());
    });

    test('addLocation attaches a location to a resource', () async {
      await repo.addVideo(NewVideoInput(resource: resource, meta: meta));
      final result = await repo.addLocation(
        'vid-1',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/vids/a.mp4', storageType: StorageType.localFs),
      );
      expect(result, isA<Success<ResourceLocation, AppFailure>>());

      final detail = (await repo.getVideo('vid-1') as Success<VideoDetail, AppFailure>).value;
      expect(detail.locations, hasLength(1));
    });

    test('addLocation returns NotFoundFailure for missing resource', () async {
      final result = await repo.addLocation(
        'no-id',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/x', storageType: StorageType.localFs),
      );
      expect(result, isA<Failure<ResourceLocation, AppFailure>>());
    });

    test('removeLocation removes a location from a resource', () async {
      await repo.addVideo(NewVideoInput(resource: resource, meta: meta));
      final locResult = await repo.addLocation(
        'vid-1',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/vids/a.mp4', storageType: StorageType.localFs),
      );
      final locId = (locResult as Success<ResourceLocation, AppFailure>).value.id;
      await repo.removeLocation('vid-1', locId);

      final detail = (await repo.getVideo('vid-1') as Success<VideoDetail, AppFailure>).value;
      expect(detail.locations, isEmpty);
    });

    test('removeLocation returns NotFoundFailure for missing resource', () async {
      final result = await repo.removeLocation('no-id', 'loc-1');
      expect(result, isA<Failure<void, AppFailure>>());
    });
  });

  group('InMemoryGameRepository', () {
    late InMemoryGameRepository repo;

    setUp(() {
      repo = InMemoryGameRepository();
    });

    const resource = Resource(id: 'game-1', title: 'RPG Alpha', resourceType: ResourceType.game);
    const meta = GameMeta(resourceId: 'game-1', platform: 'Switch', store: 'eShop', developer: 'DevCo', publisher: 'PubCo', manualNotes: 'Great game');

    test('addGame stores and returns the resource', () async {
      final result = await repo.addGame(NewGameInput(resource: resource, meta: meta));
      expect(result, isA<Success<Resource, AppFailure>>());
      expect((result as Success).value, equals(resource));
    });

    test('getGame returns detail for existing id', () async {
      await repo.addGame(NewGameInput(resource: resource, meta: meta));
      final result = await repo.getGame('game-1');
      expect(result, isA<Success<GameDetail, AppFailure>>());
      final detail = (result as Success<GameDetail, AppFailure>).value;
      expect(detail.resource, equals(resource));
      expect(detail.meta, equals(meta));
      expect(detail.locations, isEmpty);
    });

    test('getGame returns NotFoundFailure for missing id', () async {
      final result = await repo.getGame('no-such-id');
      expect(result, isA<Failure<GameDetail, AppFailure>>());
      expect((result as Failure).failure, isA<NotFoundFailure>());
    });

    test('listGames returns all stored resources', () async {
      const r2 = Resource(id: 'game-2', title: 'Platformer B', resourceType: ResourceType.game);
      const m2 = GameMeta(resourceId: 'game-2');
      await repo.addGame(NewGameInput(resource: resource, meta: meta));
      await repo.addGame(NewGameInput(resource: r2, meta: m2));

      final result = await repo.listGames();
      expect((result as Success).value, hasLength(2));
    });

    test('searchGames filters by title', () async {
      const r2 = Resource(id: 'game-2', title: 'Platformer B', resourceType: ResourceType.game);
      const m2 = GameMeta(resourceId: 'game-2');
      await repo.addGame(NewGameInput(resource: resource, meta: meta));
      await repo.addGame(NewGameInput(resource: r2, meta: m2));

      final result = await repo.searchGames('rpg');
      final list = (result as Success<List<Resource>, AppFailure>).value;
      expect(list, hasLength(1));
      expect(list.first.title, 'RPG Alpha');
    });

    test('updateGame updates title and meta fields', () async {
      await repo.addGame(NewGameInput(resource: resource, meta: meta));
      final result = await repo.updateGame(
        'game-1',
        const UpdateGameInput(title: 'RPG Alpha Updated', platform: 'PC'),
      );
      expect(result, isA<Success<Resource, AppFailure>>());
      expect((result as Success<Resource, AppFailure>).value.title, 'RPG Alpha Updated');

      final detail = (await repo.getGame('game-1') as Success<GameDetail, AppFailure>).value;
      expect(detail.meta.platform, 'PC');
      expect(detail.meta.store, 'eShop'); // unchanged
    });

    test('updateGame returns NotFoundFailure for missing id', () async {
      final result = await repo.updateGame('no-id', const UpdateGameInput(title: 'x'));
      expect(result, isA<Failure<Resource, AppFailure>>());
    });

    test('deleteGame removes the resource', () async {
      await repo.addGame(NewGameInput(resource: resource, meta: meta));
      await repo.deleteGame('game-1');
      final result = await repo.getGame('game-1');
      expect(result, isA<Failure<GameDetail, AppFailure>>());
    });

    test('addLocation attaches a location to a resource', () async {
      await repo.addGame(NewGameInput(resource: resource, meta: meta));
      final result = await repo.addLocation(
        'game-1',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/games/a', storageType: StorageType.localFs),
      );
      expect(result, isA<Success<ResourceLocation, AppFailure>>());

      final detail = (await repo.getGame('game-1') as Success<GameDetail, AppFailure>).value;
      expect(detail.locations, hasLength(1));
    });

    test('addLocation returns NotFoundFailure for missing resource', () async {
      final result = await repo.addLocation(
        'no-id',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/x', storageType: StorageType.localFs),
      );
      expect(result, isA<Failure<ResourceLocation, AppFailure>>());
    });

    test('removeLocation removes a location from a resource', () async {
      await repo.addGame(NewGameInput(resource: resource, meta: meta));
      final locResult = await repo.addLocation(
        'game-1',
        const NewLocationInput(deviceId: 'dev-1', pathOrUrl: '/games/a', storageType: StorageType.localFs),
      );
      final locId = (locResult as Success<ResourceLocation, AppFailure>).value.id;
      await repo.removeLocation('game-1', locId);

      final detail = (await repo.getGame('game-1') as Success<GameDetail, AppFailure>).value;
      expect(detail.locations, isEmpty);
    });

    test('removeLocation returns NotFoundFailure for missing resource', () async {
      final result = await repo.removeLocation('no-id', 'loc-1');
      expect(result, isA<Failure<void, AppFailure>>());
    });
  });
}
