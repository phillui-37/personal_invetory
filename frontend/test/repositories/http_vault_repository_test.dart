import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:personal_inventory_frontend/config/app_config.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/vault.dart';
import 'package:personal_inventory_frontend/repositories/http_vault_repository.dart';

const _config = AppConfig(baseUrl: 'http://localhost:3000', apiKey: 'test-key');

void main() {
  group('HttpVaultRepository', () {
    test('getStatus calls GET /api/v1/vault/status', () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response(
          jsonEncode({'initialized': true, 'unlocked': false}),
          200,
        );
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.getStatus();

      expect(method, 'GET');
      expect(requestedUri.path, '/api/v1/vault/status');
      expect(result, isA<Success<VaultStatus, AppFailure>>());
      final status = (result as Success<VaultStatus, AppFailure>).value;
      expect(status.initialized, true);
      expect(status.unlocked, false);
    });

    test('initialize calls POST /api/v1/vault/initialize with body', () async {
      late Uri requestedUri;
      late String method;
      late String body;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        body = request.body;
        return http.Response('', 200);
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.initialize('secret');

      expect(method, 'POST');
      expect(requestedUri.path, '/api/v1/vault/initialize');
      expect(jsonDecode(body), {'master_password': 'secret'});
      expect(result, isA<Success<void, AppFailure>>());
    });

    test('unlock calls POST /api/v1/vault/unlock', () async {
      late Uri requestedUri;
      late String body;
      final client = MockClient((request) async {
        requestedUri = request.url;
        body = request.body;
        return http.Response('', 200);
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      await repo.unlock('pass');

      expect(requestedUri.path, '/api/v1/vault/unlock');
      expect(jsonDecode(body), {'master_password': 'pass'});
    });

    test('lock calls POST /api/v1/vault/lock', () async {
      late Uri requestedUri;
      final client = MockClient((request) async {
        requestedUri = request.url;
        return http.Response('', 200);
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      await repo.lock();

      expect(requestedUri.path, '/api/v1/vault/lock');
    });

    test('listPlatforms calls GET /api/v1/vault/platforms', () async {
      late Uri requestedUri;
      final client = MockClient((request) async {
        requestedUri = request.url;
        return http.Response(
          jsonEncode({'platforms': ['steam', 'dlsite']}),
          200,
        );
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.listPlatforms();

      expect(requestedUri.path, '/api/v1/vault/platforms');
      expect(result, isA<Success<List<String>, AppFailure>>());
      expect(
        (result as Success<List<String>, AppFailure>).value,
        ['steam', 'dlsite'],
      );
    });

    test('maps non-200 to ServerFailure', () async {
      final client = MockClient((_) async => http.Response('', 500));
      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.getStatus();

      expect(result, isA<Failure<VaultStatus, AppFailure>>());
      expect(
        (result as Failure<VaultStatus, AppFailure>).failure,
        const ServerFailure(500),
      );
    });

    test('maps network error to NetworkFailure', () async {
      final client = MockClient((_) async => throw Exception('no connection'));
      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.getStatus();

      expect(result, isA<Failure<VaultStatus, AppFailure>>());
      expect(
        (result as Failure<VaultStatus, AppFailure>).failure,
        isA<NetworkFailure>(),
      );
    });

    test('sends authorization header', () async {
      late Map<String, String> headers;
      final client = MockClient((request) async {
        headers = request.headers;
        return http.Response(
          jsonEncode({'initialized': false, 'unlocked': false}),
          200,
        );
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      await repo.getStatus();

      expect(headers['authorization'], 'Bearer test-key');
    });
  });
}
