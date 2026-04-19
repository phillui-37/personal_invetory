import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/device/device_bloc.dart';
import 'package:personal_inventory_frontend/models/device.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/device_repository.dart';

final _device1 = Device(
  id: 'u1',
  deviceId: 'desktop-home',
  deviceName: 'Home Desktop',
  linkedAt: DateTime.utc(2024),
  locationCount: 2,
  isCurrent: true,
);

final _device2 = Device(
  id: 'u2',
  deviceId: 'laptop-work',
  deviceName: 'laptop-work',
  linkedAt: DateTime.utc(2024),
  locationCount: 0,
  isCurrent: false,
);

void main() {
  group('DeviceBloc', () {
    blocTest<DeviceBloc, DeviceState>(
      'emits loading then list on LoadDevices success',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onList: () async => Success([_device1, _device2]),
      )),
      act: (bloc) => bloc.add(const LoadDevices()),
      expect: () => [
        const DeviceLoading(),
        DeviceListLoaded([_device1, _device2]),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then error on LoadDevices failure',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onList: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const LoadDevices()),
      expect: () => const [
        DeviceLoading(),
        DeviceError(NetworkFailure('offline')),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then current loaded on LoadCurrentDevice success',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onCurrent: () async => Success(_device1),
      )),
      act: (bloc) => bloc.add(const LoadCurrentDevice()),
      expect: () => [
        const DeviceLoading(),
        DeviceCurrentLoaded(_device1),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then error on LoadCurrentDevice not-found',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onCurrent: () async =>
            const Failure(NotFoundFailure('current device')),
      )),
      act: (bloc) => bloc.add(const LoadCurrentDevice()),
      expect: () => const [
        DeviceLoading(),
        DeviceError(NotFoundFailure('current device')),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then operation success on RegisterDevice',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onRegister: (_, __) async => Success(_device1),
      )),
      act: (bloc) => bloc.add(
        const RegisterDevice(deviceId: 'desktop-home', deviceName: 'Home'),
      ),
      expect: () => const [
        DeviceLoading(),
        DeviceOperationSuccess(DeviceOperationType.register),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then error on RegisterDevice failure',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onRegister: (_, __) async => const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const RegisterDevice(deviceId: 'x')),
      expect: () => const [
        DeviceLoading(),
        DeviceError(ServerFailure(500)),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then operation success on DelinkDevice',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onDelink: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const DelinkDevice('laptop-work')),
      expect: () => const [
        DeviceLoading(),
        DeviceOperationSuccess(DeviceOperationType.delink),
      ],
    );

    blocTest<DeviceBloc, DeviceState>(
      'emits loading then error on DelinkDevice failure',
      build: () => DeviceBloc(_FakeDeviceRepo(
        onDelink: (_) async => const Failure(ServerFailure(422)),
      )),
      act: (bloc) => bloc.add(const DelinkDevice('desktop-home')),
      expect: () => const [
        DeviceLoading(),
        DeviceError(ServerFailure(422)),
      ],
    );
  });
}

final class _FakeDeviceRepo implements DeviceRepository {
  _FakeDeviceRepo({
    this.onList,
    this.onCurrent,
    this.onRegister,
    this.onDelink,
  });

  final Future<Result<List<Device>, AppFailure>> Function()? onList;
  final Future<Result<Device, AppFailure>> Function()? onCurrent;
  final Future<Result<Device, AppFailure>> Function(
    String deviceId,
    String? deviceName,
  )? onRegister;
  final Future<Result<void, AppFailure>> Function(String)? onDelink;

  @override
  Future<Result<List<Device>, AppFailure>> listDevices() =>
      onList?.call() ?? Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<Device, AppFailure>> currentDevice() =>
      onCurrent?.call() ?? Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<Device, AppFailure>> registerDevice({
    required String deviceId,
    String? deviceName,
  }) =>
      onRegister?.call(deviceId, deviceName) ??
      Future.value(const Failure(ServerFailure(500)));

  @override
  Future<Result<void, AppFailure>> delinkDevice(String deviceId) =>
      onDelink?.call(deviceId) ??
      Future.value(const Failure(ServerFailure(500)));
}
