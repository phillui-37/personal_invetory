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

final class BatchImportEntry extends Equatable {
  const BatchImportEntry({
    required this.title,
    this.author,
    this.isbn,
    this.publisher,
    this.language,
    this.fileFormat,
    this.filePath,
  });

  final String title;
  final String? author;
  final String? isbn;
  final String? publisher;
  final String? language;
  final String? fileFormat;
  final String? filePath;

  @override
  List<Object?> get props => [
    title,
    author,
    isbn,
    publisher,
    language,
    fileFormat,
    filePath,
  ];
}

final class BatchImportFailureItem extends Equatable {
  const BatchImportFailureItem({
    required this.index,
    required this.error,
  });

  final int index;
  final String error;

  @override
  List<Object?> get props => [index, error];
}

final class BatchImportResult extends Equatable {
  const BatchImportResult({
    required this.succeeded,
    required this.failed,
  });

  final List<String> succeeded;
  final List<BatchImportFailureItem> failed;

  @override
  List<Object?> get props => [succeeded, failed];
}
