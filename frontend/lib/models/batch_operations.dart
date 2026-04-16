import 'package:equatable/equatable.dart';

enum BatchOperationType {
  importResources,
  updateMetadata,
  copyMetadata,
}

final class BatchImportRequest extends Equatable {
  const BatchImportRequest({
    required this.paths,
    required this.recursive,
  });

  final List<String> paths;
  final bool recursive;

  @override
  List<Object?> get props => [paths, recursive];
}

final class BatchMetadataUpdateRequest extends Equatable {
  const BatchMetadataUpdateRequest({
    required this.resourceIds,
    required this.fields,
  });

  final List<String> resourceIds;
  final Map<String, String> fields;

  @override
  List<Object?> get props => [resourceIds, fields];
}

final class BatchMetadataCopyRequest extends Equatable {
  const BatchMetadataCopyRequest({
    required this.sourceResourceId,
    required this.targetResourceIds,
  });

  final String sourceResourceId;
  final List<String> targetResourceIds;

  @override
  List<Object?> get props => [sourceResourceId, targetResourceIds];
}

final class BatchOperationItemResult extends Equatable {
  const BatchOperationItemResult({
    required this.itemKey,
    required this.success,
    this.errorMessage,
  });

  final String itemKey;
  final bool success;
  final String? errorMessage;

  @override
  List<Object?> get props => [itemKey, success, errorMessage];
}

final class BatchOperationResponse extends Equatable {
  const BatchOperationResponse({
    required this.type,
    required this.results,
  });

  final BatchOperationType type;
  final List<BatchOperationItemResult> results;

  @override
  List<Object?> get props => [type, results];
}
