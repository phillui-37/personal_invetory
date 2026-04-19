import '../models/device.dart';
import '../models/failures.dart';
import '../models/result.dart';

abstract interface class DeviceRepository {
  Future<Result<List<Device>, AppFailure>> listDevices();
  Future<Result<Device, AppFailure>> currentDevice();
  Future<Result<Device, AppFailure>> registerDevice({
    required String deviceId,
    String? deviceName,
  });
  Future<Result<void, AppFailure>> delinkDevice(String deviceId);
}
