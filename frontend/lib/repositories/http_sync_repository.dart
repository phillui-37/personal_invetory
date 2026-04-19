import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/failures.dart';
import '../models/result.dart';
import '../models/sync.dart';
import 'sync_repository.dart';

class HttpSyncRepository implements SyncRepository {
  HttpSyncRepository({required this.config, http.Client? client})
      : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
        'Authorization': config.authorizationHeader,
        'Content-Type': 'application/json',
      };

  @override
  Future<Result<List<PlatformStatus>, AppFailure>> getEcosystemStatus() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/ecosystem/status');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final body = jsonDecode(response.body) as Map<String, dynamic>;
        final platforms = (body['platforms'] as List<dynamic>)
            .map((e) => PlatformStatus.fromJson(e as Map<String, dynamic>))
            .toList();
        return Success(platforms);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<List<SyncJob>, AppFailure>> listPlatformSyncs(
      String platform) async {
    try {
      final uri = Uri.parse(
          '${config.baseUrl}/api/v1/ecosystem/$platform/syncs');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final body = jsonDecode(response.body) as Map<String, dynamic>;
        final jobs = (body['jobs'] as List<dynamic>)
            .map((e) => SyncJob.fromJson(e as Map<String, dynamic>))
            .toList();
        return Success(jobs);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<SyncJob, AppFailure>> triggerSync(String platform) async {
    try {
      final uri = Uri.parse(
          '${config.baseUrl}/api/v1/ecosystem/$platform/sync');
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode({'credentials': null}),
      );
      if (response.statusCode == 202) {
        final body = jsonDecode(response.body) as Map<String, dynamic>;
        // Reconstruct a minimal SyncJob from the trigger response.
        return Success(SyncJob(
          id: body['job_id'] as String,
          platform: body['platform'] as String,
          status: SyncJobStatus.fromString(body['status'] as String),
          itemsFound: 0,
          itemsCreated: 0,
          itemsSkipped: 0,
          itemsFailed: 0,
          createdAt: DateTime.now(),
        ));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }
}
