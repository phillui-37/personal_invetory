import 'package:equatable/equatable.dart';

enum SyncJobStatus {
  pending, running, completed, failed;

  static SyncJobStatus fromString(String s) => switch (s) {
    'Running' => running,
    'Completed' => completed,
    'Failed' => failed,
    _ => pending,
  };
}

final class SyncJob extends Equatable {
  const SyncJob({
    required this.id,
    required this.platform,
    required this.status,
    this.startedAt,
    this.completedAt,
    required this.itemsFound,
    required this.itemsCreated,
    required this.itemsSkipped,
    required this.itemsFailed,
    this.errorMessage,
    required this.createdAt,
  });

  factory SyncJob.fromJson(Map<String, dynamic> json) => SyncJob(
    id: json['id'] as String,
    platform: json['platform'] as String,
    status: SyncJobStatus.fromString(json['status'] as String),
    startedAt: json['started_at'] != null
        ? DateTime.parse(json['started_at'] as String)
        : null,
    completedAt: json['completed_at'] != null
        ? DateTime.parse(json['completed_at'] as String)
        : null,
    itemsFound: json['items_found'] as int,
    itemsCreated: json['items_created'] as int,
    itemsSkipped: json['items_skipped'] as int,
    itemsFailed: json['items_failed'] as int,
    errorMessage: json['error_message'] as String?,
    createdAt: DateTime.parse(json['created_at'] as String),
  );

  final String id;
  final String platform;
  final SyncJobStatus status;
  final DateTime? startedAt;
  final DateTime? completedAt;
  final int itemsFound;
  final int itemsCreated;
  final int itemsSkipped;
  final int itemsFailed;
  final String? errorMessage;
  final DateTime createdAt;

  @override
  List<Object?> get props => [
    id, platform, status, startedAt, completedAt,
    itemsFound, itemsCreated, itemsSkipped, itemsFailed,
    errorMessage, createdAt,
  ];
}

final class PlatformStatus extends Equatable {
  const PlatformStatus({required this.platform, this.lastSync});

  factory PlatformStatus.fromJson(Map<String, dynamic> json) => PlatformStatus(
    platform: json['platform'] as String,
    lastSync: json['last_sync'] != null
        ? SyncJob.fromJson(json['last_sync'] as Map<String, dynamic>)
        : null,
  );

  final String platform;
  final SyncJob? lastSync;

  @override
  List<Object?> get props => [platform, lastSync];
}
