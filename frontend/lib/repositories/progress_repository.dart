import '../models/failures.dart';
import '../models/progress.dart';
import '../models/resources.dart';
import '../models/result.dart';

abstract interface class ProgressRepository {
  Future<Result<ResourceProgress?, AppFailure>> getProgress(
    ResourceType resourceType,
    String resourceId,
  );

  Future<Result<ResourceProgress, AppFailure>> upsertProgress(
    ResourceType resourceType,
    String resourceId,
    double progress, {
    String? notes,
  });
}
