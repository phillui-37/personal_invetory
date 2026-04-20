import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/failures.dart';
import '../models/progress.dart';
import '../models/resources.dart';
import '../models/result.dart';
import 'progress_repository.dart';

class HttpProgressRepository implements ProgressRepository {
  HttpProgressRepository({required this.config, http.Client? client})
      : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
        'Authorization': config.authorizationHeader,
        'Content-Type': 'application/json',
      };

  @override
  Future<Result<ResourceProgress?, AppFailure>> getProgress(
    ResourceType resourceType,
    String resourceId,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/inventory/${_pathSegment(resourceType)}/${Uri.encodeComponent(resourceId)}/progress',
      );
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(
          ResourceProgress.fromJson(jsonDecode(response.body) as Map<String, dynamic>),
        );
      }
      if (response.statusCode == 404) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<ResourceProgress, AppFailure>> upsertProgress(
    ResourceType resourceType,
    String resourceId,
    double progress, {
    String? notes,
  }) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/inventory/${_pathSegment(resourceType)}/${Uri.encodeComponent(resourceId)}/progress',
      );
      final response = await _client.patch(
        uri,
        headers: _headers,
        body: jsonEncode(<String, dynamic>{'progress': progress, 'notes': notes}),
      );
      if (response.statusCode == 200) {
        return Success(
          ResourceProgress.fromJson(jsonDecode(response.body) as Map<String, dynamic>),
        );
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  String _pathSegment(ResourceType resourceType) => switch (resourceType) {
        ResourceType.ebook => 'ebooks',
        ResourceType.webReader => 'web-readers',
        ResourceType.image => 'images',
        ResourceType.video => 'videos',
        ResourceType.game => 'games',
      };
}
