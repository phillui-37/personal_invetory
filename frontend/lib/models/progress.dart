import 'package:equatable/equatable.dart';

final class ResourceProgress extends Equatable {
  const ResourceProgress({
    required this.resourceId,
    required this.progress,
    this.notes,
    required this.updatedAt,
  });

  final String resourceId;
  final double progress;
  final String? notes;
  final DateTime updatedAt;

  factory ResourceProgress.fromJson(Map<String, dynamic> json) => ResourceProgress(
        resourceId: json['resource_id'] as String,
        progress: (json['progress'] as num).toDouble(),
        notes: json['notes'] as String?,
        updatedAt: DateTime.parse(json['updated_at'] as String),
      );

  Map<String, dynamic> toJson() => <String, dynamic>{
        'resource_id': resourceId,
        'progress': progress,
        'notes': notes,
        'updated_at': _toApiTimestamp(updatedAt),
      };

  @override
  List<Object?> get props => [resourceId, progress, notes, updatedAt];

  static String _toApiTimestamp(DateTime value) =>
      value.toUtc().toIso8601String().replaceFirst('.000Z', 'Z');
}
