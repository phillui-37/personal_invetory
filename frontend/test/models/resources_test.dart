import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/resources.dart';

void main() {
  test('Resource supports value equality', () {
    const one = Resource(id: 'r1', title: 'Book', resourceType: ResourceType.ebook);
    const two = Resource(id: 'r1', title: 'Book', resourceType: ResourceType.ebook);

    expect(one, equals(two));
    expect(one.hashCode, equals(two.hashCode));
  });

  test('ResourceLocation supports value equality', () {
    const one = ResourceLocation(
      id: 'loc1',
      resourceId: 'r1',
      deviceId: 'usb-A',
      pathOrUrl: '/mnt/books/book.epub',
      storageType: StorageType.portable,
    );
    const two = ResourceLocation(
      id: 'loc1',
      resourceId: 'r1',
      deviceId: 'usb-A',
      pathOrUrl: '/mnt/books/book.epub',
      storageType: StorageType.portable,
    );

    expect(one, equals(two));
  });

  test('Failure hierarchy supports value equality', () {
    const a = ValidationFailure(field: 'title', message: 'required');
    const b = ValidationFailure(field: 'title', message: 'required');

    expect(a, equals(b));
  });

  test('ResourceType exposes all phase 3 resource kinds', () {
    expect(
      ResourceType.values.map((value) => value.name),
      orderedEquals(['ebook', 'webReader', 'image', 'video', 'game']),
    );
  });

  test('ImageMeta supports value equality', () {
    const a = ImageMeta(resourceId: 'r1', width: 800, height: 600, fileFormat: 'png');
    const b = ImageMeta(resourceId: 'r1', width: 800, height: 600, fileFormat: 'png');
    expect(a, equals(b));
  });

  test('VideoMeta supports value equality', () {
    const a = VideoMeta(resourceId: 'r1', durationSecs: 120, fileFormat: 'mp4');
    const b = VideoMeta(resourceId: 'r1', durationSecs: 120, fileFormat: 'mp4');
    expect(a, equals(b));
  });

  test('GameMeta supports value equality', () {
    const a = GameMeta(resourceId: 'r1', platform: 'Switch', store: 'eShop');
    const b = GameMeta(resourceId: 'r1', platform: 'Switch', store: 'eShop');
    expect(a, equals(b));
  });

  test('ImageDetail holds resource, meta and locations', () {
    const detail = ImageDetail(
      resource: Resource(id: 'r1', title: 'pic', resourceType: ResourceType.image),
      meta: ImageMeta(resourceId: 'r1'),
      locations: [],
    );
    expect(detail.resource.resourceType, ResourceType.image);
  });

  test('VideoDetail holds resource, meta and locations', () {
    const detail = VideoDetail(
      resource: Resource(id: 'r1', title: 'vid', resourceType: ResourceType.video),
      meta: VideoMeta(resourceId: 'r1'),
      locations: [],
    );
    expect(detail.resource.resourceType, ResourceType.video);
  });

  test('GameDetail holds resource, meta and locations', () {
    const detail = GameDetail(
      resource: Resource(id: 'r1', title: 'game', resourceType: ResourceType.game),
      meta: GameMeta(resourceId: 'r1'),
      locations: [],
    );
    expect(detail.resource.resourceType, ResourceType.game);
  });
}
