import 'package:equatable/equatable.dart';

class Device extends Equatable {
  const Device({
    required this.id,
    required this.deviceId,
    required this.deviceName,
    required this.linkedAt,
    this.delinkedAt,
    required this.locationCount,
    required this.isCurrent,
  });

  final String id;
  final String deviceId;
  final String deviceName;
  final DateTime linkedAt;
  final DateTime? delinkedAt;
  final int locationCount;
  final bool isCurrent;

  bool get isActive => delinkedAt == null;

  factory Device.fromJson(Map<String, dynamic> json) => Device(
        id: json['id'] as String,
        deviceId: json['device_id'] as String,
        deviceName: json['device_name'] as String,
        linkedAt: DateTime.parse(json['linked_at'] as String),
        delinkedAt: json['delinked_at'] != null
            ? DateTime.parse(json['delinked_at'] as String)
            : null,
        locationCount: json['location_count'] as int,
        isCurrent: json['is_current'] as bool,
      );

  @override
  List<Object?> get props => [
        id,
        deviceId,
        deviceName,
        linkedAt,
        delinkedAt,
        locationCount,
        isCurrent,
      ];
}
