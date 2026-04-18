import 'package:equatable/equatable.dart';

enum DedupWarningStatus {
  pending, dismissed, merged;

  static DedupWarningStatus fromString(String s) => switch (s) {
    'pending' => pending,
    'dismissed' => dismissed,
    'merged' => merged,
    _ => pending,
  };
}

final class DedupWarning extends Equatable {
  const DedupWarning({
    required this.id,
    required this.resourceIdA,
    required this.resourceIdB,
    required this.similarityScore,
    required this.status,
  });

  factory DedupWarning.fromJson(Map<String, dynamic> json) => DedupWarning(
    id: json['id'] as String,
    resourceIdA: json['resource_id_a'] as String,
    resourceIdB: json['resource_id_b'] as String,
    similarityScore: (json['similarity_score'] as num).toDouble(),
    status: DedupWarningStatus.fromString(json['status'] as String),
  );

  final String id;
  final String resourceIdA;
  final String resourceIdB;
  final double similarityScore;
  final DedupWarningStatus status;

  @override
  List<Object?> get props => [id, resourceIdA, resourceIdB, similarityScore, status];
}

final class MergeInput {
  const MergeInput({required this.keepId, required this.discardId});

  final String keepId;
  final String discardId;

  Map<String, dynamic> toJson() => {
    'keep_id': keepId,
    'discard_id': discardId,
  };
}
