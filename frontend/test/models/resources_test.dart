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
}
