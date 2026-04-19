import '../models/failures.dart';
import '../models/result.dart';
import '../models/sync.dart';

abstract interface class SyncRepository {
  Future<Result<List<PlatformStatus>, AppFailure>> getEcosystemStatus();
  Future<Result<List<SyncJob>, AppFailure>> listPlatformSyncs(String platform);
  Future<Result<SyncJob, AppFailure>> triggerSync(String platform);
}
