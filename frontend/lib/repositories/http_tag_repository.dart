import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/failures.dart';
import '../models/resources.dart';
import '../models/result.dart';
import '../models/tag.dart';
import 'tag_repository.dart';

class HttpTagRepository implements TagRepository {
  HttpTagRepository({required this.config, http.Client? client})
      : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
        'Authorization': config.authorizationHeader,
        'Content-Type': 'application/json',
      };

  @override
  Future<Result<List<Tag>, AppFailure>> listTags() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/tags');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(_parseTags(response.body));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<Tag, AppFailure>> createTag(String name) async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/tags');
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(<String, dynamic>{'name': name}),
      );
      if (response.statusCode == 201) {
        return Success(Tag.fromJson(jsonDecode(response.body) as Map<String, dynamic>));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> deleteTag(String id) async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/tags/${Uri.encodeComponent(id)}');
      final response = await _client.delete(uri, headers: _headers);
      if (response.statusCode == 204) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<List<Tag>, AppFailure>> tagsForResource(
    ResourceType resourceType,
    String resourceId,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/inventory/${_pathSegment(resourceType)}/${Uri.encodeComponent(resourceId)}/tags',
      );
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(_parseTags(response.body));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> attachTag(
    ResourceType resourceType,
    String resourceId,
    String tagId,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/inventory/${_pathSegment(resourceType)}/${Uri.encodeComponent(resourceId)}/tags',
      );
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(<String, dynamic>{'tag_id': tagId}),
      );
      if (response.statusCode == 200) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> detachTag(
    ResourceType resourceType,
    String resourceId,
    String tagId,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/inventory/${_pathSegment(resourceType)}/${Uri.encodeComponent(resourceId)}/tags/${Uri.encodeComponent(tagId)}',
      );
      final response = await _client.delete(uri, headers: _headers);
      if (response.statusCode == 204) {
        return const Success(null);
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

  List<Tag> _parseTags(String body) {
    final list = jsonDecode(body) as List<dynamic>;
    return list.map((entry) => Tag.fromJson(entry as Map<String, dynamic>)).toList();
  }
}
