import '../models/batch_operations.dart';
import '../models/failures.dart';
import '../models/result.dart';

abstract interface class BatchOperationRepository {
  Future<Result<BatchOperationResponse, AppFailure>> batchImport(
    BatchImportRequest request,
  );

  Future<Result<BatchOperationResponse, AppFailure>> batchUpdateMetadata(
    BatchMetadataUpdateRequest request,
  );

  Future<Result<BatchOperationResponse, AppFailure>> batchCopyMetadata(
    BatchMetadataCopyRequest request,
  );
}
