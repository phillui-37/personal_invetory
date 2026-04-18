import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/failures.dart';
import '../models/resources.dart';
import '../models/result.dart';
import 'notification_repository.dart';

class HttpNotificationRepository implements NotificationRepository {
  HttpNotificationRepository({required this.config, http.Client? client})
    : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
    'Authorization': config.authorizationHeader,
    'Content-Type': 'application/json',
  };

  @override
  Future<Result<List<AppNotification>, AppFailure>> listNotifications({
    bool unreadOnly = false,
  }) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/notifications',
      ).replace(queryParameters: unreadOnly ? {'unread_only': 'true'} : null);
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final List<dynamic> body =
            jsonDecode(response.body) as List<dynamic>;
        return Success(
          body
              .cast<Map<String, dynamic>>()
              .map(AppNotification.fromJson)
              .toList(),
        );
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> markRead(String id) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/notifications/$id/read',
      );
      final response = await _client.post(uri, headers: _headers);
      if (response.statusCode == 200 || response.statusCode == 204) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Stream<AppNotification> watchNotifications() {
    late final Future<StreamSubscription<String>?> connection;
    late final StreamController<AppNotification> controller;
    controller = StreamController<AppNotification>(
      onListen: () {
        connection = _connectSse(controller);
      },
      onCancel: () async {
        final subscription = await connection;
        await subscription?.cancel();
      },
    );
    return controller.stream;
  }

  Future<StreamSubscription<String>?> _connectSse(
    StreamController<AppNotification> controller,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/notifications/stream',
      );
      final request = http.Request('GET', uri)
        ..headers.addAll({
          ..._headers,
          'Accept': 'text/event-stream',
          'Cache-Control': 'no-cache',
        });
      final streamedResponse = await _client.send(request);
      if (streamedResponse.statusCode != 200) {
        controller.addError(
          ServerFailure(streamedResponse.statusCode),
        );
        await controller.close();
        return null;
      }

      final buffer = StringBuffer();
      return streamedResponse.stream
          .transform(utf8.decoder)
          .listen(
            (chunk) {
              buffer.write(chunk);
              final text = buffer.toString();
              final lines = text.split('\n');
              // Keep last incomplete line in buffer
              buffer
                ..clear()
                ..write(lines.last);
              for (final line in lines.take(lines.length - 1)) {
                if (line.startsWith('data:')) {
                  final data = line.substring(5).trim();
                  if (data.isEmpty) continue;
                  try {
                    final json = jsonDecode(data) as Map<String, dynamic>;
                    controller.add(_mapStreamNotification(json));
                  } catch (_) {
                    // skip malformed events
                  }
                }
              }
            },
            onError: (Object e) {
              controller.addError(NetworkFailure(e.toString()));
            },
            onDone: () => controller.close(),
          );
    } catch (e) {
      controller.addError(NetworkFailure(e.toString()));
      await controller.close();
      return null;
    }
  }

  AppNotification _mapStreamNotification(Map<String, dynamic> json) {
    if (json.containsKey('created_at') && json.containsKey('read')) {
      return AppNotification.fromJson(json);
    }

    return AppNotification(
      id: json['id'] as String,
      resourceId: json['resource_id'] as String,
      message: json['message'] as String,
      createdAt: DateTime.now().toUtc(),
      read: false,
    );
  }
}
