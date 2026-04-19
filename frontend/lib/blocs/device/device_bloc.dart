import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/device.dart';
import '../../models/failures.dart';
import '../../models/result.dart';
import '../../repositories/device_repository.dart';

// ── Events ──────────────────────────────────────────────────────

sealed class DeviceEvent extends Equatable {
  const DeviceEvent();
}

final class LoadDevices extends DeviceEvent {
  const LoadDevices();
  @override
  List<Object?> get props => [];
}

final class LoadCurrentDevice extends DeviceEvent {
  const LoadCurrentDevice();
  @override
  List<Object?> get props => [];
}

final class RegisterDevice extends DeviceEvent {
  const RegisterDevice({required this.deviceId, this.deviceName});
  final String deviceId;
  final String? deviceName;
  @override
  List<Object?> get props => [deviceId, deviceName];
}

final class DelinkDevice extends DeviceEvent {
  const DelinkDevice(this.deviceId);
  final String deviceId;
  @override
  List<Object?> get props => [deviceId];
}

// ── States ──────────────────────────────────────────────────────

sealed class DeviceState extends Equatable {
  const DeviceState();
}

final class DeviceInitial extends DeviceState {
  const DeviceInitial();
  @override
  List<Object?> get props => [];
}

final class DeviceLoading extends DeviceState {
  const DeviceLoading();
  @override
  List<Object?> get props => [];
}

final class DeviceListLoaded extends DeviceState {
  const DeviceListLoaded(this.devices);
  final List<Device> devices;
  @override
  List<Object?> get props => [devices];
}

final class DeviceCurrentLoaded extends DeviceState {
  const DeviceCurrentLoaded(this.device);
  final Device device;
  @override
  List<Object?> get props => [device];
}

final class DeviceOperationSuccess extends DeviceState {
  const DeviceOperationSuccess(this.type);
  final DeviceOperationType type;
  @override
  List<Object?> get props => [type];
}

final class DeviceError extends DeviceState {
  const DeviceError(this.failure);
  final AppFailure failure;
  @override
  List<Object?> get props => [failure];
}

enum DeviceOperationType { register, delink }

// ── Bloc ─────────────────────────────────────────────────────────

class DeviceBloc extends Bloc<DeviceEvent, DeviceState> {
  DeviceBloc(this._repo) : super(const DeviceInitial()) {
    on<LoadDevices>(_onLoadDevices);
    on<LoadCurrentDevice>(_onLoadCurrentDevice);
    on<RegisterDevice>(_onRegisterDevice);
    on<DelinkDevice>(_onDelinkDevice);
  }

  final DeviceRepository _repo;

  Future<void> _onLoadDevices(
    LoadDevices event,
    Emitter<DeviceState> emit,
  ) async {
    emit(const DeviceLoading());
    final result = await _repo.listDevices();
    result.when(
      success: (devices) => emit(DeviceListLoaded(devices)),
      failure: (f) => emit(DeviceError(f)),
    );
  }

  Future<void> _onLoadCurrentDevice(
    LoadCurrentDevice event,
    Emitter<DeviceState> emit,
  ) async {
    emit(const DeviceLoading());
    final result = await _repo.currentDevice();
    result.when(
      success: (device) => emit(DeviceCurrentLoaded(device)),
      failure: (f) => emit(DeviceError(f)),
    );
  }

  Future<void> _onRegisterDevice(
    RegisterDevice event,
    Emitter<DeviceState> emit,
  ) async {
    emit(const DeviceLoading());
    final result = await _repo.registerDevice(
      deviceId: event.deviceId,
      deviceName: event.deviceName,
    );
    result.when(
      success: (_) =>
          emit(const DeviceOperationSuccess(DeviceOperationType.register)),
      failure: (f) => emit(DeviceError(f)),
    );
  }

  Future<void> _onDelinkDevice(
    DelinkDevice event,
    Emitter<DeviceState> emit,
  ) async {
    emit(const DeviceLoading());
    final result = await _repo.delinkDevice(event.deviceId);
    result.when(
      success: (_) =>
          emit(const DeviceOperationSuccess(DeviceOperationType.delink)),
      failure: (f) => emit(DeviceError(f)),
    );
  }
}
