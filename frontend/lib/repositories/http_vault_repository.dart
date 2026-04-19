import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/failures.dart';
import '../models/result.dart';
import '../models/vault.dart';
import 'vault_repository.dart';

class HttpVaultRepository implements VaultRepository {
  HttpVaultRepository({required this.config, http.Client? client})
    : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
    'Authorization': config.authorizationHeader,
    'Content-Type': 'application/json',
  };

  @override
  Future<Result<VaultStatus, AppFailure>> getStatus() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/vault/status');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return Success(VaultStatus.fromJson(json));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> initialize(String masterPassword) async {
    return _postVoid(
      '/api/v1/vault/initialize',
      {'master_password': masterPassword},
    );
  }

  @override
  Future<Result<void, AppFailure>> unlock(String masterPassword) async {
    return _postVoid(
      '/api/v1/vault/unlock',
      {'master_password': masterPassword},
    );
  }

  @override
  Future<Result<void, AppFailure>> lock() async {
    return _postVoid('/api/v1/vault/lock', {});
  }

  @override
  Future<Result<void, AppFailure>> storeCredential(
    StoreCredentialInput input,
  ) async {
    return _postVoid('/api/v1/vault/credentials/store', input.toJson());
  }

  @override
  Future<Result<String, AppFailure>> retrieveCredential(
    RetrieveCredentialInput input,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/vault/credentials/retrieve',
      );
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(input.toJson()),
      );
      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return Success(json['plaintext'] as String);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> deleteCredential(
    RetrieveCredentialInput input,
  ) async {
    return _postVoid('/api/v1/vault/credentials/delete', input.toJson());
  }

  @override
  Future<Result<List<String>, AppFailure>> listPlatforms() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/vault/platforms');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        final platforms = (json['platforms'] as List<dynamic>).cast<String>();
        return Success(platforms);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  Future<Result<void, AppFailure>> _postVoid(
    String path,
    Map<String, dynamic> body,
  ) async {
    try {
      final uri = Uri.parse('${config.baseUrl}$path');
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(body),
      );
      if (response.statusCode == 200 ||
          response.statusCode == 201 ||
          response.statusCode == 204) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }
}
