import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:personal_inventory_frontend/config/app_config.dart';
import 'package:personal_inventory_frontend/models/dedup.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/http_dedup_repository.dart';

const _config = AppConfig(baseUrl: 'http://localhost:3000', apiKey: 'test-key');

final _warningJson = {
  'id': 'w1',
  'resource_id_a': 'a1',
  'resource_id_b': 'b1',
  'similarity_score': 0.92,
  'status': 'pending',
};

void main() {
  group('HttpDedupRepository', () {
    test('scanDuplicates calls POST /api/v1/dedup/scan', () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response(jsonEncode([_warningJson]), 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.scanDuplicates();

      expect(method, 'POST');
      expect(requestedUri.path, '/api/v1/dedup/scan');
      expect(result, isA<Success<List<DedupWarning>, AppFailure>>());
      final warnings =
          (result as Success<List<DedupWarning>, AppFailure>).value;
      expect(warnings.length, 1);
      expect(warnings.first.id, 'w1');
    });

    test('listPendingWarnings calls GET /api/v1/dedup/warnings', () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response(jsonEncode([_warningJson]), 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.listPendingWarnings();

      expect(method, 'GET');
      expect(requestedUri.path, '/api/v1/dedup/warnings');
      expect(result, isA<Success<List<DedupWarning>, AppFailure>>());
    });

    test('dismissWarning calls POST /api/v1/dedup/warnings/:id/dismiss',
        () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response('', 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.dismissWarning('w1');

      expect(method, 'POST');
      expect(requestedUri.path, '/api/v1/dedup/warnings/w1/dismiss');
      expect(result, isA<Success<void, AppFailure>>());
    });

    test(
        'mergeResources calls POST /api/v1/dedup/warnings/:id/merge with body',
        () async {
      late Uri requestedUri;
      late String body;
      final client = MockClient((request) async {
        requestedUri = request.url;
        body = request.body;
        return http.Response('', 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.mergeResources(
        'w1',
        const MergeInput(keepId: 'a1', discardId: 'b1'),
      );

      expect(requestedUri.path, '/api/v1/dedup/warnings/w1/merge');
      expect(jsonDecode(body), {'keep_id': 'a1', 'discard_id': 'b1'});
      expect(result, isA<Success<void, AppFailure>>());
    });

    test('maps non-200 to ServerFailure', () async {
      final client = MockClient((_) async => http.Response('', 500));
      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.listPendingWarnings();

      expect(result, isA<Failure<List<DedupWarning>, AppFailure>>());
    });

    test('maps network error to NetworkFailure', () async {
      final client =
          MockClient((_) async => throw Exception('no connection'));
      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.scanDuplicates();

      expect(result, isA<Failure<List<DedupWarning>, AppFailure>>());
      expect(
        (result as Failure<List<DedupWarning>, AppFailure>).failure,
        isA<NetworkFailure>(),
      );
    });

    test('sends authorization header', () async {
      late Map<String, String> headers;
      final client = MockClient((request) async {
        headers = request.headers;
        return http.Response(jsonEncode([]), 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      await repo.listPendingWarnings();

      expect(headers['authorization'], 'Bearer test-key');
    });
  });
}
