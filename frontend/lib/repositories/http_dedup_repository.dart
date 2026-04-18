import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/dedup.dart';
import '../models/failures.dart';
import '../models/result.dart';
import 'dedup_repository.dart';

class HttpDedupRepository implements DedupRepository {
  HttpDedupRepository({required this.config, http.Client? client})
    : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
    'Authorization': config.authorizationHeader,
    'Content-Type': 'application/json',
  };

  @override
  Future<Result<List<DedupWarning>, AppFailure>> scanDuplicates() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/dedup/scan');
      final response = await _client.post(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(_parseWarnings(response.body));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<List<DedupWarning>, AppFailure>> listPendingWarnings() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/dedup/warnings');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(_parseWarnings(response.body));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> dismissWarning(String id) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/dedup/warnings/$id/dismiss',
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
  Future<Result<void, AppFailure>> mergeResources(
    String warningId,
    MergeInput input,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/dedup/warnings/$warningId/merge',
      );
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(input.toJson()),
      );
      if (response.statusCode == 200 || response.statusCode == 204) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  List<DedupWarning> _parseWarnings(String body) {
    final list = jsonDecode(body) as List<dynamic>;
    return list
        .cast<Map<String, dynamic>>()
        .map(DedupWarning.fromJson)
        .toList();
  }
}
