import '../models/failures.dart';
import '../models/result.dart';
import '../models/vault.dart';

abstract interface class VaultRepository {
  Future<Result<VaultStatus, AppFailure>> getStatus();
  Future<Result<void, AppFailure>> initialize(String masterPassword);
  Future<Result<void, AppFailure>> unlock(String masterPassword);
  Future<Result<void, AppFailure>> lock();
  Future<Result<void, AppFailure>> storeCredential(StoreCredentialInput input);
  Future<Result<String, AppFailure>> retrieveCredential(RetrieveCredentialInput input);
  Future<Result<void, AppFailure>> deleteCredential(RetrieveCredentialInput input);
  Future<Result<List<String>, AppFailure>> listPlatforms();
}
