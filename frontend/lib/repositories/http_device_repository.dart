import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/device.dart';
import '../models/failures.dart';
import '../models/result.dart';
import 'device_repository.dart';

class HttpDeviceRepository implements DeviceRepository {
  HttpDeviceRepository({required this.config, http.Client? client})
      : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
        'Authorization': config.authorizationHeader,
        'Content-Type': 'application/json',
      };

  @override
  Future<Result<List<Device>, AppFailure>> listDevices() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/devices');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final list = jsonDecode(response.body) as List<dynamic>;
        return Success(
          list
              .map((e) => Device.fromJson(e as Map<String, dynamic>))
              .toList(),
        );
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<Device, AppFailure>> currentDevice() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/devices/current');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(
          Device.fromJson(jsonDecode(response.body) as Map<String, dynamic>),
        );
      }
      if (response.statusCode == 404) {
        return const Failure(NotFoundFailure('current device'));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<Device, AppFailure>> registerDevice({
    required String deviceId,
    String? deviceName,
  }) async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/devices/register');
      final body = <String, dynamic>{'device_id': deviceId};
      if (deviceName != null) body['device_name'] = deviceName;
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(body),
      );
      if (response.statusCode == 201) {
        return Success(
          Device.fromJson(jsonDecode(response.body) as Map<String, dynamic>),
        );
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> delinkDevice(String deviceId) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/devices/${Uri.encodeComponent(deviceId)}/delink',
      );
      final response = await _client.post(uri, headers: _headers, body: '{}');
      if (response.statusCode == 200) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }
}
