import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/device.dart';

void main() {
  group('Device.fromJson', () {
    test('parses active device', () {
      final json = {
        'id': 'uuid-1',
        'device_id': 'desktop-home',
        'device_name': 'Home Desktop',
        'linked_at': '2024-01-01T00:00:00.000Z',
        'location_count': 3,
        'is_current': true,
      };
      final device = Device.fromJson(json);
      expect(device.id, 'uuid-1');
      expect(device.deviceId, 'desktop-home');
      expect(device.deviceName, 'Home Desktop');
      expect(device.locationCount, 3);
      expect(device.isCurrent, isTrue);
      expect(device.isActive, isTrue);
    });

    test('parses delinked device', () {
      final json = {
        'id': 'uuid-2',
        'device_id': 'laptop-work',
        'device_name': 'Work Laptop',
        'linked_at': '2024-01-01T00:00:00.000Z',
        'delinked_at': '2024-06-01T00:00:00.000Z',
        'location_count': 0,
        'is_current': false,
      };
      final device = Device.fromJson(json);
      expect(device.delinkedAt, isNotNull);
      expect(device.isActive, isFalse);
      expect(device.isCurrent, isFalse);
    });

    test('Equatable props', () {
      final a = Device.fromJson({
        'id': 'u1',
        'device_id': 'dev',
        'device_name': 'Dev',
        'linked_at': '2024-01-01T00:00:00.000Z',
        'location_count': 0,
        'is_current': false,
      });
      final b = Device.fromJson({
        'id': 'u1',
        'device_id': 'dev',
        'device_name': 'Dev',
        'linked_at': '2024-01-01T00:00:00.000Z',
        'location_count': 0,
        'is_current': false,
      });
      expect(a, equals(b));
    });
  });
}
