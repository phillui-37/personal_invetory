import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/batch_operations.dart';
import '../models/failures.dart';
import '../models/resources.dart';
import '../models/result.dart';
import 'batch_operation_repository.dart';

class HttpBatchOperationRepository implements BatchOperationRepository {
  HttpBatchOperationRepository({
    required this.config,
    required this.resourceType,
    http.Client? client,
  }) : _client = client ?? http.Client();

  final AppConfig config;
  final ResourceType resourceType;
  final http.Client _client;

  Map<String, String> get _headers => {
        'Authorization': config.authorizationHeader,
        'Content-Type': 'application/json',
      };

  String get _typePath => switch (resourceType) {
        ResourceType.ebook => 'ebooks',
        ResourceType.webReader => 'web-readers',
        ResourceType.image => 'images',
        ResourceType.video => 'videos',
        ResourceType.game => 'games',
      };

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchImport(
    BatchImportRequest request,
  ) async {
    // Batch import is not yet a real backend endpoint; return stub success.
    return Success(
      BatchOperationResponse(
        type: BatchOperationType.importResources,
        results: request.paths
            .map(
              (p) => BatchOperationItemResult(itemKey: p, success: true),
            )
            .toList(),
      ),
    );
  }

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchUpdateMetadata(
    BatchMetadataUpdateRequest request,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/inventory/$_typePath/batch-update',
      );
      final response = await _client.patch(
        uri,
        headers: _headers,
        body: jsonEncode(<String, dynamic>{
          'ids': request.resourceIds,
          'fields': request.fields,
        }),
      );
      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        final failed = (json['failed'] as List<dynamic>? ?? [])
            .cast<Map<String, dynamic>>();
        final failedIds = failed.map((f) => f['id'] as String).toSet();
        final results = request.resourceIds
            .map(
              (id) => BatchOperationItemResult(
                itemKey: id,
                success: !failedIds.contains(id),
                errorMessage: failedIds.contains(id)
                    ? failed
                        .firstWhere((f) => f['id'] == id)['reason'] as String?
                    : null,
              ),
            )
            .toList();
        return Success(
          BatchOperationResponse(
            type: BatchOperationType.updateMetadata,
            results: results,
          ),
        );
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<BatchOperationResponse, AppFailure>> batchCopyMetadata(
    BatchMetadataCopyRequest request,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/inventory/$_typePath/batch-copy-meta',
      );
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(<String, dynamic>{
          'source_id': request.sourceResourceId,
          'target_ids': request.targetResourceIds,
        }),
      );
      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        final failed = (json['failed'] as List<dynamic>? ?? [])
            .cast<Map<String, dynamic>>();
        final failedIds = failed.map((f) => f['id'] as String).toSet();
        final results = request.targetResourceIds
            .map(
              (id) => BatchOperationItemResult(
                itemKey: id,
                success: !failedIds.contains(id),
                errorMessage: failedIds.contains(id)
                    ? failed
                        .firstWhere((f) => f['id'] == id)['reason'] as String?
                    : null,
              ),
            )
            .toList();
        return Success(
          BatchOperationResponse(
            type: BatchOperationType.copyMetadata,
            results: results,
          ),
        );
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }
}
