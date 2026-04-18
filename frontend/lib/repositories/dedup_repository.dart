import '../models/dedup.dart';
import '../models/failures.dart';
import '../models/result.dart';

abstract interface class DedupRepository {
  Future<Result<List<DedupWarning>, AppFailure>> scanDuplicates();
  Future<Result<List<DedupWarning>, AppFailure>> listPendingWarnings();
  Future<Result<void, AppFailure>> dismissWarning(String id);
  Future<Result<void, AppFailure>> mergeResources(String warningId, MergeInput input);
}
