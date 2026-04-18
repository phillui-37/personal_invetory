import 'dart:async';
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:personal_inventory_frontend/config/app_config.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/resources.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/http_notification_repository.dart';

class _StreamingClient extends http.BaseClient {
  _StreamingClient(this.responseFactory);

  final Future<http.StreamedResponse> Function(http.BaseRequest request)
  responseFactory;

  @override
  Future<http.StreamedResponse> send(http.BaseRequest request) {
    return responseFactory(request);
  }
}

void main() {
  group('HttpNotificationRepository', () {
    test('listNotifications calls /api/v1/notifications', () async {
      late Uri requestedUri;
      final client = MockClient((request) async {
        requestedUri = request.url;
        return http.Response(
          jsonEncode([
            {
              'id': 'n1',
              'resource_id': 'r1',
              'message': 'hello',
              'created_at': '2025-01-01T00:00:00Z',
              'read': false,
            },
          ]),
          200,
        );
      });

      final repo = HttpNotificationRepository(
        config: const AppConfig(baseUrl: 'http://localhost:3000', apiKey: 'secret'),
        client: client,
      );

      final result = await repo.listNotifications();

      expect(requestedUri.path, '/api/v1/notifications');
      expect(result, isA<Success<List<AppNotification>, AppFailure>>());
    });

    test('markRead posts to /api/v1/notifications/:id/read', () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response('', 204);
      });

      final repo = HttpNotificationRepository(
        config: const AppConfig(baseUrl: 'http://localhost:3000', apiKey: 'secret'),
        client: client,
      );

      final result = await repo.markRead('n1');

      expect(method, 'POST');
      expect(requestedUri.path, '/api/v1/notifications/n1/read');
      expect(result, isA<Success<void, AppFailure>>());
    });

    test('maps non-200 list response to ServerFailure', () async {
      final client = MockClient((_) async => http.Response('', 500));
      final repo = HttpNotificationRepository(
        config: const AppConfig(baseUrl: 'http://localhost:3000', apiKey: 'secret'),
        client: client,
      );

      final result = await repo.listNotifications();

      expect(result, isA<Failure<List<AppNotification>, AppFailure>>());
      expect(
        (result as Failure<List<AppNotification>, AppFailure>).failure,
        const ServerFailure(500),
      );
    });

    test('watchNotifications cancels the SSE stream on unsubscribe', () async {
      var cancelled = false;
      final responseController = StreamController<List<int>>(
        onCancel: () => cancelled = true,
      );
      final client = _StreamingClient(
        (_) async => http.StreamedResponse(responseController.stream, 200),
      );
      final repo = HttpNotificationRepository(
        config: const AppConfig(baseUrl: 'http://localhost:3000', apiKey: 'secret'),
        client: client,
      );

      final subscription = repo.watchNotifications().listen((_) {});
      await Future<void>.delayed(Duration.zero);
      await subscription.cancel();
      await Future<void>.delayed(Duration.zero);

      expect(cancelled, isTrue);
    });
  });
}
