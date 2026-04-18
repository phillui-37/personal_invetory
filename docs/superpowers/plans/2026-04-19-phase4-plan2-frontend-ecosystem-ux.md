# Phase 4 Plan 2: Frontend Ecosystem UX — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the frontend screens, BLoCs, repositories, and models for vault management and dedup review — the two ecosystem features that have working backend APIs.

**Architecture:** New models (`vault.dart`, `dedup.dart`) following existing Equatable+fromJson pattern. Abstract interface repositories with HTTP implementations using `http` package + `MockClient` for tests. BLoCs following sealed event/state pattern with `bloc_test`. Widget tests with `MultiBlocProvider` + fake repositories. New "Ecosystem" tab in bottom navigation as hub for vault and dedup screens.

**Tech Stack:** Flutter, flutter_bloc, equatable, http, bloc_test, MockClient

**Scope Note:** EcosystemBloc, Ecosystem Settings Screen, and Sync Dashboard are **deferred** — no backend API endpoints exist for ecosystem sync operations yet. This plan covers only what has working backend APIs: vault (8 endpoints) and dedup (4 endpoints).

**Backend API routes (for reference):**
- Vault: `GET /api/v1/vault/status`, `POST /api/v1/vault/initialize`, `POST /api/v1/vault/unlock`, `POST /api/v1/vault/lock`, `POST /api/v1/vault/credentials/store`, `POST /api/v1/vault/credentials/retrieve`, `POST /api/v1/vault/credentials/delete`, `GET /api/v1/vault/platforms`
- Dedup: `POST /api/v1/dedup/scan`, `GET /api/v1/dedup/warnings`, `POST /api/v1/dedup/warnings/:id/dismiss`, `POST /api/v1/dedup/warnings/:id/merge`
- Auth: All routes require `Authorization: Bearer <api_key>` header

**Existing patterns to follow:**
- BLoC: see `lib/blocs/ebook/ebook_bloc.dart` — sealed events/states, `Equatable`, `result.when()` dispatching
- HTTP repo: see `lib/repositories/http_notification_repository.dart` — `AppConfig` + optional `http.Client`, try-catch, `ServerFailure`/`NetworkFailure`
- Fake repo: see `test/support/fake_repositories.dart` — configurable `Result` fields, call counters
- BLoC tests: see `test/blocs/ebook/ebook_bloc_test.dart` — `blocTest<>()`, private `_FakeRepo` with callback fields
- HTTP tests: see `test/repositories/http_notification_repository_test.dart` — `MockClient`, URI/method assertions
- Models: see `lib/models/failures.dart` — `Equatable`, `final class`, `fromJson`/`toJson`

---

## File Structure

### New Files

| File | Responsibility |
|------|---------------|
| `lib/models/vault.dart` | VaultStatus, StoreCredentialInput, RetrieveCredentialInput |
| `lib/models/dedup.dart` | DedupWarning, DedupWarningStatus, MergeInput |
| `lib/repositories/vault_repository.dart` | Abstract interface for vault operations |
| `lib/repositories/dedup_repository.dart` | Abstract interface for dedup operations |
| `lib/repositories/http_vault_repository.dart` | HTTP implementation calling backend vault API |
| `lib/repositories/http_dedup_repository.dart` | HTTP implementation calling backend dedup API |
| `lib/blocs/vault/vault_bloc.dart` | VaultBloc: events, states, handlers |
| `lib/blocs/dedup/dedup_bloc.dart` | DedupBloc: events, states, handlers |
| `lib/screens/vault_screen.dart` | Vault status, initialize, unlock/lock UI |
| `lib/screens/dedup_review_screen.dart` | Dedup warnings list with dismiss/merge |
| `lib/screens/ecosystem_screen.dart` | Hub screen with cards linking to vault + dedup |
| `test/models/vault_test.dart` | VaultStatus fromJson tests |
| `test/models/dedup_test.dart` | DedupWarning fromJson, MergeInput toJson tests |
| `test/blocs/vault/vault_bloc_test.dart` | VaultBloc state transition tests |
| `test/blocs/dedup/dedup_bloc_test.dart` | DedupBloc state transition tests |
| `test/repositories/http_vault_repository_test.dart` | HTTP call verification tests |
| `test/repositories/http_dedup_repository_test.dart` | HTTP call verification tests |
| `test/screens/vault_screen_test.dart` | Widget tests for vault screen |
| `test/screens/dedup_review_screen_test.dart` | Widget tests for dedup review screen |

### Modified Files

| File | Change |
|------|--------|
| `lib/main.dart` | Add VaultBloc + DedupBloc providers, add "Ecosystem" tab |
| `test/support/fake_repositories.dart` | Add FakeVaultRepository + FakeDedupRepository |

---

### Task 1: Vault & Dedup Domain Models

**Files:**
- Create: `lib/models/vault.dart`
- Create: `lib/models/dedup.dart`
- Create: `test/models/vault_test.dart`
- Create: `test/models/dedup_test.dart`

- [ ] **Step 1: Write failing tests for VaultStatus**

Create `test/models/vault_test.dart`:

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/vault.dart';

void main() {
  group('VaultStatus', () {
    test('fromJson parses initialized and unlocked', () {
      final json = {'initialized': true, 'unlocked': false};
      final status = VaultStatus.fromJson(json);
      expect(status.initialized, true);
      expect(status.unlocked, false);
    });

    test('equality based on fields', () {
      const a = VaultStatus(initialized: true, unlocked: true);
      const b = VaultStatus(initialized: true, unlocked: true);
      const c = VaultStatus(initialized: false, unlocked: true);
      expect(a, equals(b));
      expect(a, isNot(equals(c)));
    });
  });

  group('StoreCredentialInput', () {
    test('toJson produces correct keys', () {
      const input = StoreCredentialInput(
        platform: 'steam',
        credentialType: 'api_key',
        plaintext: 'secret123',
      );
      expect(input.toJson(), {
        'platform': 'steam',
        'credential_type': 'api_key',
        'plaintext': 'secret123',
      });
    });
  });

  group('RetrieveCredentialInput', () {
    test('toJson produces correct keys', () {
      const input = RetrieveCredentialInput(
        platform: 'steam',
        credentialType: 'api_key',
      );
      expect(input.toJson(), {
        'platform': 'steam',
        'credential_type': 'api_key',
      });
    });
  });
}
```

- [ ] **Step 2: Write failing tests for DedupWarning and MergeInput**

Create `test/models/dedup_test.dart`:

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/models/dedup.dart';

void main() {
  group('DedupWarningStatus', () {
    test('fromString parses known statuses', () {
      expect(DedupWarningStatus.fromString('pending'), DedupWarningStatus.pending);
      expect(DedupWarningStatus.fromString('dismissed'), DedupWarningStatus.dismissed);
      expect(DedupWarningStatus.fromString('merged'), DedupWarningStatus.merged);
    });

    test('fromString defaults to pending for unknown', () {
      expect(DedupWarningStatus.fromString('unknown'), DedupWarningStatus.pending);
    });
  });

  group('DedupWarning', () {
    test('fromJson parses all fields', () {
      final json = {
        'id': 'w1',
        'resource_id_a': 'a1',
        'resource_id_b': 'b1',
        'similarity_score': 0.92,
        'status': 'pending',
      };
      final warning = DedupWarning.fromJson(json);
      expect(warning.id, 'w1');
      expect(warning.resourceIdA, 'a1');
      expect(warning.resourceIdB, 'b1');
      expect(warning.similarityScore, 0.92);
      expect(warning.status, DedupWarningStatus.pending);
    });

    test('fromJson handles integer similarity_score', () {
      final json = {
        'id': 'w1',
        'resource_id_a': 'a1',
        'resource_id_b': 'b1',
        'similarity_score': 1,
        'status': 'merged',
      };
      final warning = DedupWarning.fromJson(json);
      expect(warning.similarityScore, 1.0);
    });

    test('equality based on fields', () {
      const a = DedupWarning(
        id: 'w1', resourceIdA: 'a1', resourceIdB: 'b1',
        similarityScore: 0.92, status: DedupWarningStatus.pending,
      );
      const b = DedupWarning(
        id: 'w1', resourceIdA: 'a1', resourceIdB: 'b1',
        similarityScore: 0.92, status: DedupWarningStatus.pending,
      );
      expect(a, equals(b));
    });
  });

  group('MergeInput', () {
    test('toJson produces correct keys', () {
      const input = MergeInput(keepId: 'a1', discardId: 'b1');
      expect(input.toJson(), {'keep_id': 'a1', 'discard_id': 'b1'});
    });
  });
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd frontend && flutter test test/models/vault_test.dart test/models/dedup_test.dart`
Expected: FAIL — source files don't exist

- [ ] **Step 4: Implement VaultStatus model**

Create `lib/models/vault.dart`:

```dart
import 'package:equatable/equatable.dart';

final class VaultStatus extends Equatable {
  const VaultStatus({required this.initialized, required this.unlocked});

  factory VaultStatus.fromJson(Map<String, dynamic> json) => VaultStatus(
    initialized: json['initialized'] as bool,
    unlocked: json['unlocked'] as bool,
  );

  final bool initialized;
  final bool unlocked;

  @override
  List<Object?> get props => [initialized, unlocked];
}

final class StoreCredentialInput {
  const StoreCredentialInput({
    required this.platform,
    required this.credentialType,
    required this.plaintext,
  });

  final String platform;
  final String credentialType;
  final String plaintext;

  Map<String, dynamic> toJson() => {
    'platform': platform,
    'credential_type': credentialType,
    'plaintext': plaintext,
  };
}

final class RetrieveCredentialInput {
  const RetrieveCredentialInput({
    required this.platform,
    required this.credentialType,
  });

  final String platform;
  final String credentialType;

  Map<String, dynamic> toJson() => {
    'platform': platform,
    'credential_type': credentialType,
  };
}
```

- [ ] **Step 5: Implement DedupWarning model**

Create `lib/models/dedup.dart`:

```dart
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
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd frontend && flutter test test/models/vault_test.dart test/models/dedup_test.dart`
Expected: All tests PASS

- [ ] **Step 7: Commit**

```bash
git add lib/models/vault.dart lib/models/dedup.dart test/models/vault_test.dart test/models/dedup_test.dart
git commit -m "feat(frontend): add vault and dedup domain models"
```

---

### Task 2: Repository Interfaces + Fake Implementations

**Files:**
- Create: `lib/repositories/vault_repository.dart`
- Create: `lib/repositories/dedup_repository.dart`
- Modify: `test/support/fake_repositories.dart`

- [ ] **Step 1: Create VaultRepository interface**

Create `lib/repositories/vault_repository.dart`:

```dart
import '../models/failures.dart';
import '../models/result.dart';
import '../models/vault.dart';

abstract interface class VaultRepository {
  Future<Result<VaultStatus, AppFailure>> getStatus();
  Future<Result<void, AppFailure>> initialize(String masterPassword);
  Future<Result<void, AppFailure>> unlock(String masterPassword);
  Future<Result<void, AppFailure>> lock();
  Future<Result<void, AppFailure>> storeCredential(StoreCredentialInput input);
  Future<Result<String, AppFailure>> retrieveCredential(
    RetrieveCredentialInput input,
  );
  Future<Result<void, AppFailure>> deleteCredential(
    RetrieveCredentialInput input,
  );
  Future<Result<List<String>, AppFailure>> listPlatforms();
}
```

- [ ] **Step 2: Create DedupRepository interface**

Create `lib/repositories/dedup_repository.dart`:

```dart
import '../models/dedup.dart';
import '../models/failures.dart';
import '../models/result.dart';

abstract interface class DedupRepository {
  Future<Result<List<DedupWarning>, AppFailure>> scanDuplicates();
  Future<Result<List<DedupWarning>, AppFailure>> listPendingWarnings();
  Future<Result<void, AppFailure>> dismissWarning(String id);
  Future<Result<void, AppFailure>> mergeResources(
    String warningId,
    MergeInput input,
  );
}
```

- [ ] **Step 3: Add FakeVaultRepository and FakeDedupRepository**

Add these imports at top of `test/support/fake_repositories.dart`:

```dart
import 'package:personal_inventory_frontend/models/dedup.dart';
import 'package:personal_inventory_frontend/models/vault.dart';
import 'package:personal_inventory_frontend/repositories/dedup_repository.dart';
import 'package:personal_inventory_frontend/repositories/vault_repository.dart';
```

Append to end of `test/support/fake_repositories.dart`:

```dart
class FakeVaultRepository implements VaultRepository {
  FakeVaultRepository({
    this.statusResult = const Success(
      VaultStatus(initialized: false, unlocked: false),
    ),
    this.initializeResult = const Success(null),
    this.unlockResult = const Success(null),
    this.lockResult = const Success(null),
    this.storeCredentialResult = const Success(null),
    this.retrieveCredentialResult = const Success('plaintext'),
    this.deleteCredentialResult = const Success(null),
    this.listPlatformsResult = const Success([]),
  });

  Result<VaultStatus, AppFailure> statusResult;
  Result<void, AppFailure> initializeResult;
  Result<void, AppFailure> unlockResult;
  Result<void, AppFailure> lockResult;
  Result<void, AppFailure> storeCredentialResult;
  Result<String, AppFailure> retrieveCredentialResult;
  Result<void, AppFailure> deleteCredentialResult;
  Result<List<String>, AppFailure> listPlatformsResult;

  int statusCalls = 0;
  int initializeCalls = 0;
  int unlockCalls = 0;
  int lockCalls = 0;
  int storeCredentialCalls = 0;
  int retrieveCredentialCalls = 0;
  int deleteCredentialCalls = 0;
  int listPlatformsCalls = 0;
  String? lastMasterPassword;
  StoreCredentialInput? lastStoreInput;
  RetrieveCredentialInput? lastRetrieveInput;

  @override
  Future<Result<VaultStatus, AppFailure>> getStatus() async {
    statusCalls += 1;
    return statusResult;
  }

  @override
  Future<Result<void, AppFailure>> initialize(String masterPassword) async {
    initializeCalls += 1;
    lastMasterPassword = masterPassword;
    return initializeResult;
  }

  @override
  Future<Result<void, AppFailure>> unlock(String masterPassword) async {
    unlockCalls += 1;
    lastMasterPassword = masterPassword;
    return unlockResult;
  }

  @override
  Future<Result<void, AppFailure>> lock() async {
    lockCalls += 1;
    return lockResult;
  }

  @override
  Future<Result<void, AppFailure>> storeCredential(
    StoreCredentialInput input,
  ) async {
    storeCredentialCalls += 1;
    lastStoreInput = input;
    return storeCredentialResult;
  }

  @override
  Future<Result<String, AppFailure>> retrieveCredential(
    RetrieveCredentialInput input,
  ) async {
    retrieveCredentialCalls += 1;
    lastRetrieveInput = input;
    return retrieveCredentialResult;
  }

  @override
  Future<Result<void, AppFailure>> deleteCredential(
    RetrieveCredentialInput input,
  ) async {
    deleteCredentialCalls += 1;
    return deleteCredentialResult;
  }

  @override
  Future<Result<List<String>, AppFailure>> listPlatforms() async {
    listPlatformsCalls += 1;
    return listPlatformsResult;
  }
}

class FakeDedupRepository implements DedupRepository {
  FakeDedupRepository({
    this.scanResult = const Success([]),
    this.listPendingResult = const Success([]),
    this.dismissResult = const Success(null),
    this.mergeResult = const Success(null),
  });

  Result<List<DedupWarning>, AppFailure> scanResult;
  Result<List<DedupWarning>, AppFailure> listPendingResult;
  Result<void, AppFailure> dismissResult;
  Result<void, AppFailure> mergeResult;

  int scanCalls = 0;
  int listPendingCalls = 0;
  int dismissCalls = 0;
  int mergeCalls = 0;
  String? lastDismissId;
  String? lastMergeWarningId;
  MergeInput? lastMergeInput;

  @override
  Future<Result<List<DedupWarning>, AppFailure>> scanDuplicates() async {
    scanCalls += 1;
    return scanResult;
  }

  @override
  Future<Result<List<DedupWarning>, AppFailure>> listPendingWarnings() async {
    listPendingCalls += 1;
    return listPendingResult;
  }

  @override
  Future<Result<void, AppFailure>> dismissWarning(String id) async {
    dismissCalls += 1;
    lastDismissId = id;
    return dismissResult;
  }

  @override
  Future<Result<void, AppFailure>> mergeResources(
    String warningId,
    MergeInput input,
  ) async {
    mergeCalls += 1;
    lastMergeWarningId = warningId;
    lastMergeInput = input;
    return mergeResult;
  }
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd frontend && flutter test test/models/vault_test.dart`
Expected: PASS (confirms all imports resolve)

- [ ] **Step 5: Commit**

```bash
git add lib/repositories/vault_repository.dart lib/repositories/dedup_repository.dart test/support/fake_repositories.dart
git commit -m "feat(frontend): add vault and dedup repository interfaces + fakes"
```

---

### Task 3: VaultBloc (TDD)

**Files:**
- Create: `test/blocs/vault/vault_bloc_test.dart`
- Create: `lib/blocs/vault/vault_bloc.dart`

- [ ] **Step 1: Write failing VaultBloc tests**

Create `test/blocs/vault/vault_bloc_test.dart`. Follow the pattern from `test/blocs/ebook/ebook_bloc_test.dart` — private `_FakeVaultRepo` with callback fields, `blocTest<VaultBloc, VaultState>()`.

```dart
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/vault/vault_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/vault.dart';
import 'package:personal_inventory_frontend/repositories/vault_repository.dart';

void main() {
  group('VaultBloc', () {
    blocTest<VaultBloc, VaultState>(
      'emits loading then status loaded on CheckVaultStatus success',
      build: () => VaultBloc(_FakeVaultRepo(
        onGetStatus: () async =>
            const Success(VaultStatus(initialized: true, unlocked: false)),
      )),
      act: (bloc) => bloc.add(const CheckVaultStatus()),
      expect: () => const [
        VaultLoading(),
        VaultStatusLoaded(VaultStatus(initialized: true, unlocked: false)),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then error on CheckVaultStatus failure',
      build: () => VaultBloc(_FakeVaultRepo(
        onGetStatus: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const CheckVaultStatus()),
      expect: () => const [
        VaultLoading(),
        VaultError(NetworkFailure('offline')),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then operation success on InitializeVault',
      build: () => VaultBloc(_FakeVaultRepo(
        onInitialize: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const InitializeVault('mypassword')),
      expect: () => const [
        VaultLoading(),
        VaultOperationSuccess(VaultOperationType.initialize),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then error on InitializeVault failure',
      build: () => VaultBloc(_FakeVaultRepo(
        onInitialize: (_) async => const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const InitializeVault('mypassword')),
      expect: () => const [
        VaultLoading(),
        VaultError(ServerFailure(500)),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then operation success on UnlockVault',
      build: () => VaultBloc(_FakeVaultRepo(
        onUnlock: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const UnlockVault('mypassword')),
      expect: () => const [
        VaultLoading(),
        VaultOperationSuccess(VaultOperationType.unlock),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then error on UnlockVault failure',
      build: () => VaultBloc(_FakeVaultRepo(
        onUnlock: (_) async => const Failure(ServerFailure(401)),
      )),
      act: (bloc) => bloc.add(const UnlockVault('wrong')),
      expect: () => const [
        VaultLoading(),
        VaultError(ServerFailure(401)),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then operation success on LockVault',
      build: () => VaultBloc(_FakeVaultRepo(
        onLock: () async => const Success(null),
      )),
      act: (bloc) => bloc.add(const LockVault()),
      expect: () => const [
        VaultLoading(),
        VaultOperationSuccess(VaultOperationType.lock),
      ],
    );

    blocTest<VaultBloc, VaultState>(
      'emits loading then platforms loaded on LoadPlatforms',
      build: () => VaultBloc(_FakeVaultRepo(
        onListPlatforms: () async => const Success(['steam', 'dlsite']),
      )),
      act: (bloc) => bloc.add(const LoadPlatforms()),
      expect: () => const [
        VaultLoading(),
        VaultPlatformsLoaded(['steam', 'dlsite']),
      ],
    );
  });
}

final class _FakeVaultRepo implements VaultRepository {
  _FakeVaultRepo({
    this.onGetStatus,
    this.onInitialize,
    this.onUnlock,
    this.onLock,
    this.onStoreCredential,
    this.onRetrieveCredential,
    this.onDeleteCredential,
    this.onListPlatforms,
  });

  final Future<Result<VaultStatus, AppFailure>> Function()? onGetStatus;
  final Future<Result<void, AppFailure>> Function(String)? onInitialize;
  final Future<Result<void, AppFailure>> Function(String)? onUnlock;
  final Future<Result<void, AppFailure>> Function()? onLock;
  final Future<Result<void, AppFailure>> Function(StoreCredentialInput)?
      onStoreCredential;
  final Future<Result<String, AppFailure>> Function(RetrieveCredentialInput)?
      onRetrieveCredential;
  final Future<Result<void, AppFailure>> Function(RetrieveCredentialInput)?
      onDeleteCredential;
  final Future<Result<List<String>, AppFailure>> Function()? onListPlatforms;

  @override
  Future<Result<VaultStatus, AppFailure>> getStatus() async =>
      onGetStatus?.call() ?? const Failure(ServerFailure(500));
  @override
  Future<Result<void, AppFailure>> initialize(String p) async =>
      onInitialize?.call(p) ?? const Failure(ServerFailure(500));
  @override
  Future<Result<void, AppFailure>> unlock(String p) async =>
      onUnlock?.call(p) ?? const Failure(ServerFailure(500));
  @override
  Future<Result<void, AppFailure>> lock() async =>
      onLock?.call() ?? const Failure(ServerFailure(500));
  @override
  Future<Result<void, AppFailure>> storeCredential(StoreCredentialInput i) async =>
      onStoreCredential?.call(i) ?? const Failure(ServerFailure(500));
  @override
  Future<Result<String, AppFailure>> retrieveCredential(RetrieveCredentialInput i) async =>
      onRetrieveCredential?.call(i) ?? const Failure(ServerFailure(500));
  @override
  Future<Result<void, AppFailure>> deleteCredential(RetrieveCredentialInput i) async =>
      onDeleteCredential?.call(i) ?? const Failure(ServerFailure(500));
  @override
  Future<Result<List<String>, AppFailure>> listPlatforms() async =>
      onListPlatforms?.call() ?? const Failure(ServerFailure(500));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd frontend && flutter test test/blocs/vault/vault_bloc_test.dart`
Expected: FAIL — `vault_bloc.dart` doesn't exist

- [ ] **Step 3: Implement VaultBloc**

Create `lib/blocs/vault/vault_bloc.dart`. Follow pattern from `lib/blocs/ebook/ebook_bloc.dart` — sealed events extending Equatable, sealed states extending Equatable, `result.when()` dispatch.

```dart
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/failures.dart';
import '../../models/result.dart';
import '../../models/vault.dart';
import '../../repositories/vault_repository.dart';

// --- Events ---

sealed class VaultEvent extends Equatable {
  const VaultEvent();
}

final class CheckVaultStatus extends VaultEvent {
  const CheckVaultStatus();
  @override
  List<Object?> get props => [];
}

final class InitializeVault extends VaultEvent {
  const InitializeVault(this.masterPassword);
  final String masterPassword;
  @override
  List<Object?> get props => [masterPassword];
}

final class UnlockVault extends VaultEvent {
  const UnlockVault(this.masterPassword);
  final String masterPassword;
  @override
  List<Object?> get props => [masterPassword];
}

final class LockVault extends VaultEvent {
  const LockVault();
  @override
  List<Object?> get props => [];
}

final class LoadPlatforms extends VaultEvent {
  const LoadPlatforms();
  @override
  List<Object?> get props => [];
}

// --- States ---

enum VaultOperationType { initialize, unlock, lock }

sealed class VaultState extends Equatable {
  const VaultState();
}

final class VaultInitial extends VaultState {
  const VaultInitial();
  @override
  List<Object?> get props => [];
}

final class VaultLoading extends VaultState {
  const VaultLoading();
  @override
  List<Object?> get props => [];
}

final class VaultStatusLoaded extends VaultState {
  const VaultStatusLoaded(this.status);
  final VaultStatus status;
  @override
  List<Object?> get props => [status];
}

final class VaultPlatformsLoaded extends VaultState {
  const VaultPlatformsLoaded(this.platforms);
  final List<String> platforms;
  @override
  List<Object?> get props => [platforms];
}

final class VaultOperationSuccess extends VaultState {
  const VaultOperationSuccess(this.operationType);
  final VaultOperationType operationType;
  @override
  List<Object?> get props => [operationType];
}

final class VaultError extends VaultState {
  const VaultError(this.failure);
  final AppFailure failure;
  @override
  List<Object?> get props => [failure];
}

// --- Bloc ---

final class VaultBloc extends Bloc<VaultEvent, VaultState> {
  VaultBloc(this._repository) : super(const VaultInitial()) {
    on<CheckVaultStatus>(_onCheckStatus);
    on<InitializeVault>(_onInitialize);
    on<UnlockVault>(_onUnlock);
    on<LockVault>(_onLock);
    on<LoadPlatforms>(_onLoadPlatforms);
  }

  final VaultRepository _repository;

  Future<void> _onCheckStatus(
    CheckVaultStatus event, Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.getStatus();
    result.when(
      success: (status) => emit(VaultStatusLoaded(status)),
      failure: (failure) => emit(VaultError(failure)),
    );
  }

  Future<void> _onInitialize(
    InitializeVault event, Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.initialize(event.masterPassword);
    _emitVoidResult(result, emit, VaultOperationType.initialize);
  }

  Future<void> _onUnlock(
    UnlockVault event, Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.unlock(event.masterPassword);
    _emitVoidResult(result, emit, VaultOperationType.unlock);
  }

  Future<void> _onLock(
    LockVault event, Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.lock();
    _emitVoidResult(result, emit, VaultOperationType.lock);
  }

  Future<void> _onLoadPlatforms(
    LoadPlatforms event, Emitter<VaultState> emit,
  ) async {
    emit(const VaultLoading());
    final result = await _repository.listPlatforms();
    result.when(
      success: (platforms) => emit(VaultPlatformsLoaded(platforms)),
      failure: (failure) => emit(VaultError(failure)),
    );
  }

  void _emitVoidResult(
    Result<void, AppFailure> result,
    Emitter<VaultState> emit,
    VaultOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(VaultOperationSuccess(operationType)),
      failure: (failure) => emit(VaultError(failure)),
    );
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd frontend && flutter test test/blocs/vault/vault_bloc_test.dart`
Expected: All 8 tests PASS

- [ ] **Step 5: Commit**

```bash
git add lib/blocs/vault/vault_bloc.dart test/blocs/vault/vault_bloc_test.dart
git commit -m "feat(frontend): add VaultBloc with TDD"
```

---

### Task 4: DedupBloc (TDD)

**Files:**
- Create: `test/blocs/dedup/dedup_bloc_test.dart`
- Create: `lib/blocs/dedup/dedup_bloc.dart`

- [ ] **Step 1: Write failing DedupBloc tests**

Create `test/blocs/dedup/dedup_bloc_test.dart`:

```dart
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/dedup/dedup_bloc.dart';
import 'package:personal_inventory_frontend/models/dedup.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/dedup_repository.dart';

void main() {
  const warning1 = DedupWarning(
    id: 'w1', resourceIdA: 'a1', resourceIdB: 'b1',
    similarityScore: 0.92, status: DedupWarningStatus.pending,
  );
  const warning2 = DedupWarning(
    id: 'w2', resourceIdA: 'a2', resourceIdB: 'b2',
    similarityScore: 0.88, status: DedupWarningStatus.pending,
  );

  group('DedupBloc', () {
    blocTest<DedupBloc, DedupState>(
      'emits loading then warnings loaded on LoadWarnings success',
      build: () => DedupBloc(_FakeDedupRepo(
        onListPending: () async => const Success([warning1, warning2]),
      )),
      act: (bloc) => bloc.add(const LoadWarnings()),
      expect: () => const [
        DedupLoading(),
        DedupWarningsLoaded([warning1, warning2]),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then error on LoadWarnings failure',
      build: () => DedupBloc(_FakeDedupRepo(
        onListPending: () async => const Failure(NetworkFailure('offline')),
      )),
      act: (bloc) => bloc.add(const LoadWarnings()),
      expect: () => const [
        DedupLoading(),
        DedupError(NetworkFailure('offline')),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then warnings loaded on ScanDuplicates success',
      build: () => DedupBloc(_FakeDedupRepo(
        onScan: () async => const Success([warning1]),
      )),
      act: (bloc) => bloc.add(const ScanDuplicates()),
      expect: () => const [
        DedupLoading(),
        DedupWarningsLoaded([warning1]),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then operation success on DismissWarning',
      build: () => DedupBloc(_FakeDedupRepo(
        onDismiss: (_) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const DismissWarning('w1')),
      expect: () => const [
        DedupLoading(),
        DedupOperationSuccess(DedupOperationType.dismiss),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then error on DismissWarning failure',
      build: () => DedupBloc(_FakeDedupRepo(
        onDismiss: (_) async => const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const DismissWarning('w1')),
      expect: () => const [
        DedupLoading(),
        DedupError(ServerFailure(500)),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then operation success on MergeResources',
      build: () => DedupBloc(_FakeDedupRepo(
        onMerge: (_, __) async => const Success(null),
      )),
      act: (bloc) => bloc.add(const MergeResources(
        warningId: 'w1', keepId: 'a1', discardId: 'b1',
      )),
      expect: () => const [
        DedupLoading(),
        DedupOperationSuccess(DedupOperationType.merge),
      ],
    );

    blocTest<DedupBloc, DedupState>(
      'emits loading then error on MergeResources failure',
      build: () => DedupBloc(_FakeDedupRepo(
        onMerge: (_, __) async => const Failure(ServerFailure(500)),
      )),
      act: (bloc) => bloc.add(const MergeResources(
        warningId: 'w1', keepId: 'a1', discardId: 'b1',
      )),
      expect: () => const [
        DedupLoading(),
        DedupError(ServerFailure(500)),
      ],
    );
  });
}

final class _FakeDedupRepo implements DedupRepository {
  _FakeDedupRepo({this.onScan, this.onListPending, this.onDismiss, this.onMerge});

  final Future<Result<List<DedupWarning>, AppFailure>> Function()? onScan;
  final Future<Result<List<DedupWarning>, AppFailure>> Function()? onListPending;
  final Future<Result<void, AppFailure>> Function(String)? onDismiss;
  final Future<Result<void, AppFailure>> Function(String, MergeInput)? onMerge;

  @override
  Future<Result<List<DedupWarning>, AppFailure>> scanDuplicates() async =>
      onScan?.call() ?? const Failure(ServerFailure(500));
  @override
  Future<Result<List<DedupWarning>, AppFailure>> listPendingWarnings() async =>
      onListPending?.call() ?? const Failure(ServerFailure(500));
  @override
  Future<Result<void, AppFailure>> dismissWarning(String id) async =>
      onDismiss?.call(id) ?? const Failure(ServerFailure(500));
  @override
  Future<Result<void, AppFailure>> mergeResources(String warningId, MergeInput input) async =>
      onMerge?.call(warningId, input) ?? const Failure(ServerFailure(500));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd frontend && flutter test test/blocs/dedup/dedup_bloc_test.dart`
Expected: FAIL — `dedup_bloc.dart` doesn't exist

- [ ] **Step 3: Implement DedupBloc**

Create `lib/blocs/dedup/dedup_bloc.dart`:

```dart
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/dedup.dart';
import '../../models/failures.dart';
import '../../models/result.dart';
import '../../repositories/dedup_repository.dart';

// --- Events ---

sealed class DedupEvent extends Equatable {
  const DedupEvent();
}

final class LoadWarnings extends DedupEvent {
  const LoadWarnings();
  @override
  List<Object?> get props => [];
}

final class ScanDuplicates extends DedupEvent {
  const ScanDuplicates();
  @override
  List<Object?> get props => [];
}

final class DismissWarning extends DedupEvent {
  const DismissWarning(this.warningId);
  final String warningId;
  @override
  List<Object?> get props => [warningId];
}

final class MergeResources extends DedupEvent {
  const MergeResources({
    required this.warningId,
    required this.keepId,
    required this.discardId,
  });
  final String warningId;
  final String keepId;
  final String discardId;
  @override
  List<Object?> get props => [warningId, keepId, discardId];
}

// --- States ---

enum DedupOperationType { dismiss, merge }

sealed class DedupState extends Equatable {
  const DedupState();
}

final class DedupInitial extends DedupState {
  const DedupInitial();
  @override
  List<Object?> get props => [];
}

final class DedupLoading extends DedupState {
  const DedupLoading();
  @override
  List<Object?> get props => [];
}

final class DedupWarningsLoaded extends DedupState {
  const DedupWarningsLoaded(this.warnings);
  final List<DedupWarning> warnings;
  @override
  List<Object?> get props => [warnings];
}

final class DedupOperationSuccess extends DedupState {
  const DedupOperationSuccess(this.operationType);
  final DedupOperationType operationType;
  @override
  List<Object?> get props => [operationType];
}

final class DedupError extends DedupState {
  const DedupError(this.failure);
  final AppFailure failure;
  @override
  List<Object?> get props => [failure];
}

// --- Bloc ---

final class DedupBloc extends Bloc<DedupEvent, DedupState> {
  DedupBloc(this._repository) : super(const DedupInitial()) {
    on<LoadWarnings>(_onLoadWarnings);
    on<ScanDuplicates>(_onScan);
    on<DismissWarning>(_onDismiss);
    on<MergeResources>(_onMerge);
  }

  final DedupRepository _repository;

  Future<void> _onLoadWarnings(
    LoadWarnings event, Emitter<DedupState> emit,
  ) async {
    emit(const DedupLoading());
    final result = await _repository.listPendingWarnings();
    result.when(
      success: (warnings) => emit(DedupWarningsLoaded(warnings)),
      failure: (failure) => emit(DedupError(failure)),
    );
  }

  Future<void> _onScan(
    ScanDuplicates event, Emitter<DedupState> emit,
  ) async {
    emit(const DedupLoading());
    final result = await _repository.scanDuplicates();
    result.when(
      success: (warnings) => emit(DedupWarningsLoaded(warnings)),
      failure: (failure) => emit(DedupError(failure)),
    );
  }

  Future<void> _onDismiss(
    DismissWarning event, Emitter<DedupState> emit,
  ) async {
    emit(const DedupLoading());
    final result = await _repository.dismissWarning(event.warningId);
    _emitVoidResult(result, emit, DedupOperationType.dismiss);
  }

  Future<void> _onMerge(
    MergeResources event, Emitter<DedupState> emit,
  ) async {
    emit(const DedupLoading());
    final result = await _repository.mergeResources(
      event.warningId,
      MergeInput(keepId: event.keepId, discardId: event.discardId),
    );
    _emitVoidResult(result, emit, DedupOperationType.merge);
  }

  void _emitVoidResult(
    Result<void, AppFailure> result,
    Emitter<DedupState> emit,
    DedupOperationType operationType,
  ) {
    result.when(
      success: (_) => emit(DedupOperationSuccess(operationType)),
      failure: (failure) => emit(DedupError(failure)),
    );
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd frontend && flutter test test/blocs/dedup/dedup_bloc_test.dart`
Expected: All 7 tests PASS

- [ ] **Step 5: Commit**

```bash
git add lib/blocs/dedup/dedup_bloc.dart test/blocs/dedup/dedup_bloc_test.dart
git commit -m "feat(frontend): add DedupBloc with TDD"
```

---

### Task 5: HttpVaultRepository (TDD)

**Files:**
- Create: `test/repositories/http_vault_repository_test.dart`
- Create: `lib/repositories/http_vault_repository.dart`

- [ ] **Step 1: Write failing HTTP vault repository tests**

Create `test/repositories/http_vault_repository_test.dart`. Follow pattern from `test/repositories/http_notification_repository_test.dart` — `MockClient`, assert URI paths, methods, request bodies, and response parsing.

```dart
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:personal_inventory_frontend/config/app_config.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/vault.dart';
import 'package:personal_inventory_frontend/repositories/http_vault_repository.dart';

const _config = AppConfig(baseUrl: 'http://localhost:3000', apiKey: 'test-key');

void main() {
  group('HttpVaultRepository', () {
    test('getStatus calls GET /api/v1/vault/status', () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response(
          jsonEncode({'initialized': true, 'unlocked': false}),
          200,
        );
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.getStatus();

      expect(method, 'GET');
      expect(requestedUri.path, '/api/v1/vault/status');
      expect(result, isA<Success<VaultStatus, AppFailure>>());
      final status = (result as Success<VaultStatus, AppFailure>).value;
      expect(status.initialized, true);
      expect(status.unlocked, false);
    });

    test('initialize calls POST /api/v1/vault/initialize with body', () async {
      late Uri requestedUri;
      late String method;
      late String body;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        body = request.body;
        return http.Response('', 200);
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.initialize('secret');

      expect(method, 'POST');
      expect(requestedUri.path, '/api/v1/vault/initialize');
      expect(jsonDecode(body), {'master_password': 'secret'});
      expect(result, isA<Success<void, AppFailure>>());
    });

    test('unlock calls POST /api/v1/vault/unlock', () async {
      late Uri requestedUri;
      late String body;
      final client = MockClient((request) async {
        requestedUri = request.url;
        body = request.body;
        return http.Response('', 200);
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      await repo.unlock('pass');

      expect(requestedUri.path, '/api/v1/vault/unlock');
      expect(jsonDecode(body), {'master_password': 'pass'});
    });

    test('lock calls POST /api/v1/vault/lock', () async {
      late Uri requestedUri;
      final client = MockClient((request) async {
        requestedUri = request.url;
        return http.Response('', 200);
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      await repo.lock();

      expect(requestedUri.path, '/api/v1/vault/lock');
    });

    test('listPlatforms calls GET /api/v1/vault/platforms', () async {
      late Uri requestedUri;
      final client = MockClient((request) async {
        requestedUri = request.url;
        return http.Response(
          jsonEncode({'platforms': ['steam', 'dlsite']}),
          200,
        );
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.listPlatforms();

      expect(requestedUri.path, '/api/v1/vault/platforms');
      expect(result, isA<Success<List<String>, AppFailure>>());
      expect((result as Success<List<String>, AppFailure>).value, ['steam', 'dlsite']);
    });

    test('maps non-200 to ServerFailure', () async {
      final client = MockClient((_) async => http.Response('', 500));
      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.getStatus();

      expect(result, isA<Failure<VaultStatus, AppFailure>>());
      expect(
        (result as Failure<VaultStatus, AppFailure>).failure,
        const ServerFailure(500),
      );
    });

    test('maps network error to NetworkFailure', () async {
      final client = MockClient((_) async => throw Exception('no connection'));
      final repo = HttpVaultRepository(config: _config, client: client);
      final result = await repo.getStatus();

      expect(result, isA<Failure<VaultStatus, AppFailure>>());
      expect(
        (result as Failure<VaultStatus, AppFailure>).failure,
        isA<NetworkFailure>(),
      );
    });

    test('sends authorization header', () async {
      late Map<String, String> headers;
      final client = MockClient((request) async {
        headers = request.headers;
        return http.Response(
          jsonEncode({'initialized': false, 'unlocked': false}),
          200,
        );
      });

      final repo = HttpVaultRepository(config: _config, client: client);
      await repo.getStatus();

      expect(headers['authorization'], 'Bearer test-key');
    });
  });
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd frontend && flutter test test/repositories/http_vault_repository_test.dart`
Expected: FAIL — source file doesn't exist

- [ ] **Step 3: Implement HttpVaultRepository**

Create `lib/repositories/http_vault_repository.dart`:

```dart
import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/failures.dart';
import '../models/result.dart';
import '../models/vault.dart';
import 'vault_repository.dart';

class HttpVaultRepository implements VaultRepository {
  HttpVaultRepository({required this.config, http.Client? client})
    : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
    'Authorization': config.authorizationHeader,
    'Content-Type': 'application/json',
  };

  @override
  Future<Result<VaultStatus, AppFailure>> getStatus() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/vault/status');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return Success(VaultStatus.fromJson(json));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> initialize(String masterPassword) async {
    return _postVoid(
      '/api/v1/vault/initialize',
      {'master_password': masterPassword},
    );
  }

  @override
  Future<Result<void, AppFailure>> unlock(String masterPassword) async {
    return _postVoid(
      '/api/v1/vault/unlock',
      {'master_password': masterPassword},
    );
  }

  @override
  Future<Result<void, AppFailure>> lock() async {
    return _postVoid('/api/v1/vault/lock', {});
  }

  @override
  Future<Result<void, AppFailure>> storeCredential(
    StoreCredentialInput input,
  ) async {
    return _postVoid('/api/v1/vault/credentials/store', input.toJson());
  }

  @override
  Future<Result<String, AppFailure>> retrieveCredential(
    RetrieveCredentialInput input,
  ) async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/vault/credentials/retrieve');
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(input.toJson()),
      );
      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return Success(json['plaintext'] as String);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> deleteCredential(
    RetrieveCredentialInput input,
  ) async {
    return _postVoid('/api/v1/vault/credentials/delete', input.toJson());
  }

  @override
  Future<Result<List<String>, AppFailure>> listPlatforms() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/vault/platforms');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        final platforms = (json['platforms'] as List<dynamic>).cast<String>();
        return Success(platforms);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  Future<Result<void, AppFailure>> _postVoid(
    String path,
    Map<String, dynamic> body,
  ) async {
    try {
      final uri = Uri.parse('${config.baseUrl}$path');
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(body),
      );
      if (response.statusCode == 200 || response.statusCode == 204) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd frontend && flutter test test/repositories/http_vault_repository_test.dart`
Expected: All 8 tests PASS

- [ ] **Step 5: Commit**

```bash
git add lib/repositories/http_vault_repository.dart test/repositories/http_vault_repository_test.dart
git commit -m "feat(frontend): add HttpVaultRepository with TDD"
```

---

### Task 6: HttpDedupRepository (TDD)

**Files:**
- Create: `test/repositories/http_dedup_repository_test.dart`
- Create: `lib/repositories/http_dedup_repository.dart`

- [ ] **Step 1: Write failing HTTP dedup repository tests**

Create `test/repositories/http_dedup_repository_test.dart`:

```dart
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:personal_inventory_frontend/config/app_config.dart';
import 'package:personal_inventory_frontend/models/dedup.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/http_dedup_repository.dart';

const _config = AppConfig(baseUrl: 'http://localhost:3000', apiKey: 'test-key');

final _warningJson = {
  'id': 'w1',
  'resource_id_a': 'a1',
  'resource_id_b': 'b1',
  'similarity_score': 0.92,
  'status': 'pending',
};

void main() {
  group('HttpDedupRepository', () {
    test('scanDuplicates calls POST /api/v1/dedup/scan', () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response(jsonEncode([_warningJson]), 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.scanDuplicates();

      expect(method, 'POST');
      expect(requestedUri.path, '/api/v1/dedup/scan');
      expect(result, isA<Success<List<DedupWarning>, AppFailure>>());
      final warnings = (result as Success<List<DedupWarning>, AppFailure>).value;
      expect(warnings.length, 1);
      expect(warnings.first.id, 'w1');
    });

    test('listPendingWarnings calls GET /api/v1/dedup/warnings', () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response(jsonEncode([_warningJson]), 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.listPendingWarnings();

      expect(method, 'GET');
      expect(requestedUri.path, '/api/v1/dedup/warnings');
      expect(result, isA<Success<List<DedupWarning>, AppFailure>>());
    });

    test('dismissWarning calls POST /api/v1/dedup/warnings/:id/dismiss', () async {
      late Uri requestedUri;
      late String method;
      final client = MockClient((request) async {
        requestedUri = request.url;
        method = request.method;
        return http.Response('', 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.dismissWarning('w1');

      expect(method, 'POST');
      expect(requestedUri.path, '/api/v1/dedup/warnings/w1/dismiss');
      expect(result, isA<Success<void, AppFailure>>());
    });

    test('mergeResources calls POST /api/v1/dedup/warnings/:id/merge with body', () async {
      late Uri requestedUri;
      late String body;
      final client = MockClient((request) async {
        requestedUri = request.url;
        body = request.body;
        return http.Response('', 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.mergeResources(
        'w1',
        const MergeInput(keepId: 'a1', discardId: 'b1'),
      );

      expect(requestedUri.path, '/api/v1/dedup/warnings/w1/merge');
      expect(jsonDecode(body), {'keep_id': 'a1', 'discard_id': 'b1'});
      expect(result, isA<Success<void, AppFailure>>());
    });

    test('maps non-200 to ServerFailure', () async {
      final client = MockClient((_) async => http.Response('', 500));
      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.listPendingWarnings();

      expect(result, isA<Failure<List<DedupWarning>, AppFailure>>());
    });

    test('maps network error to NetworkFailure', () async {
      final client = MockClient((_) async => throw Exception('no connection'));
      final repo = HttpDedupRepository(config: _config, client: client);
      final result = await repo.scanDuplicates();

      expect(result, isA<Failure<List<DedupWarning>, AppFailure>>());
      expect(
        (result as Failure<List<DedupWarning>, AppFailure>).failure,
        isA<NetworkFailure>(),
      );
    });

    test('sends authorization header', () async {
      late Map<String, String> headers;
      final client = MockClient((request) async {
        headers = request.headers;
        return http.Response(jsonEncode([]), 200);
      });

      final repo = HttpDedupRepository(config: _config, client: client);
      await repo.listPendingWarnings();

      expect(headers['authorization'], 'Bearer test-key');
    });
  });
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd frontend && flutter test test/repositories/http_dedup_repository_test.dart`
Expected: FAIL — source file doesn't exist

- [ ] **Step 3: Implement HttpDedupRepository**

Create `lib/repositories/http_dedup_repository.dart`:

```dart
import 'dart:convert';

import 'package:http/http.dart' as http;

import '../config/app_config.dart';
import '../models/dedup.dart';
import '../models/failures.dart';
import '../models/result.dart';
import 'dedup_repository.dart';

class HttpDedupRepository implements DedupRepository {
  HttpDedupRepository({required this.config, http.Client? client})
    : _client = client ?? http.Client();

  final AppConfig config;
  final http.Client _client;

  Map<String, String> get _headers => {
    'Authorization': config.authorizationHeader,
    'Content-Type': 'application/json',
  };

  @override
  Future<Result<List<DedupWarning>, AppFailure>> scanDuplicates() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/dedup/scan');
      final response = await _client.post(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(_parseWarnings(response.body));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<List<DedupWarning>, AppFailure>> listPendingWarnings() async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/dedup/warnings');
      final response = await _client.get(uri, headers: _headers);
      if (response.statusCode == 200) {
        return Success(_parseWarnings(response.body));
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> dismissWarning(String id) async {
    try {
      final uri = Uri.parse('${config.baseUrl}/api/v1/dedup/warnings/$id/dismiss');
      final response = await _client.post(uri, headers: _headers);
      if (response.statusCode == 200 || response.statusCode == 204) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  @override
  Future<Result<void, AppFailure>> mergeResources(
    String warningId,
    MergeInput input,
  ) async {
    try {
      final uri = Uri.parse(
        '${config.baseUrl}/api/v1/dedup/warnings/$warningId/merge',
      );
      final response = await _client.post(
        uri,
        headers: _headers,
        body: jsonEncode(input.toJson()),
      );
      if (response.statusCode == 200 || response.statusCode == 204) {
        return const Success(null);
      }
      return Failure(ServerFailure(response.statusCode));
    } catch (e) {
      return Failure(NetworkFailure(e.toString()));
    }
  }

  List<DedupWarning> _parseWarnings(String body) {
    final list = jsonDecode(body) as List<dynamic>;
    return list
        .cast<Map<String, dynamic>>()
        .map(DedupWarning.fromJson)
        .toList();
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd frontend && flutter test test/repositories/http_dedup_repository_test.dart`
Expected: All 7 tests PASS

- [ ] **Step 5: Commit**

```bash
git add lib/repositories/http_dedup_repository.dart test/repositories/http_dedup_repository_test.dart
git commit -m "feat(frontend): add HttpDedupRepository with TDD"
```

---

### Task 7: VaultScreen (TDD Widget Test)

**Files:**
- Create: `test/screens/vault_screen_test.dart`
- Create: `lib/screens/vault_screen.dart`

- [ ] **Step 1: Write failing VaultScreen widget tests**

Create `test/screens/vault_screen_test.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/vault/vault_bloc.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/models/vault.dart';
import 'package:personal_inventory_frontend/repositories/vault_repository.dart';
import 'package:personal_inventory_frontend/screens/vault_screen.dart';

import '../support/fake_repositories.dart';

Widget _buildTestWidget(VaultRepository repo) {
  return MaterialApp(
    home: BlocProvider<VaultBloc>(
      create: (_) => VaultBloc(repo),
      child: const VaultScreen(),
    ),
  );
}

void main() {
  group('VaultScreen', () {
    testWidgets('shows check status button initially', (tester) async {
      await tester.pumpWidget(_buildTestWidget(FakeVaultRepository()));
      expect(find.text('Check Vault Status'), findsOneWidget);
    });

    testWidgets('shows initialize button when vault not initialized', (tester) async {
      final repo = FakeVaultRepository(
        statusResult: const Success(
          VaultStatus(initialized: false, unlocked: false),
        ),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Check Vault Status'));
      await tester.pumpAndSettle();

      expect(find.text('Initialize Vault'), findsOneWidget);
    });

    testWidgets('shows unlock button when vault initialized but locked', (tester) async {
      final repo = FakeVaultRepository(
        statusResult: const Success(
          VaultStatus(initialized: true, unlocked: false),
        ),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Check Vault Status'));
      await tester.pumpAndSettle();

      expect(find.text('Unlock'), findsOneWidget);
    });

    testWidgets('shows lock button when vault unlocked', (tester) async {
      final repo = FakeVaultRepository(
        statusResult: const Success(
          VaultStatus(initialized: true, unlocked: true),
        ),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Check Vault Status'));
      await tester.pumpAndSettle();

      expect(find.text('Lock'), findsOneWidget);
    });

    testWidgets('shows error message on failure', (tester) async {
      final repo = FakeVaultRepository(
        statusResult: const Failure(NetworkFailure('offline')),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Check Vault Status'));
      await tester.pumpAndSettle();

      expect(find.textContaining('Error'), findsOneWidget);
    });
  });
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd frontend && flutter test test/screens/vault_screen_test.dart`
Expected: FAIL — `vault_screen.dart` doesn't exist

- [ ] **Step 3: Implement VaultScreen**

Create `lib/screens/vault_screen.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/vault/vault_bloc.dart';

class VaultScreen extends StatelessWidget {
  const VaultScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Credential Vault')),
      body: BlocConsumer<VaultBloc, VaultState>(
        listener: (context, state) {
          if (state is VaultOperationSuccess) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('${state.operationType.name} successful')),
            );
            context.read<VaultBloc>().add(const CheckVaultStatus());
          }
        },
        builder: (context, state) {
          return Padding(
            padding: const EdgeInsets.all(16),
            child: switch (state) {
              VaultInitial() => _InitialView(),
              VaultLoading() => const Center(child: CircularProgressIndicator()),
              VaultStatusLoaded(:final status) => _StatusView(status: status),
              VaultPlatformsLoaded(:final platforms) =>
                _PlatformsView(platforms: platforms),
              VaultOperationSuccess() => const SizedBox.shrink(),
              VaultError(:final failure) =>
                Center(child: Text('Error: $failure')),
            },
          );
        },
      ),
    );
  }
}

class _InitialView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: ElevatedButton(
        onPressed: () =>
            context.read<VaultBloc>().add(const CheckVaultStatus()),
        child: const Text('Check Vault Status'),
      ),
    );
  }
}

class _StatusView extends StatefulWidget {
  const _StatusView({required this.status});
  final VaultStatus status;

  @override
  State<_StatusView> createState() => _StatusViewState();
}

class _StatusViewState extends State<_StatusView> {
  final _passwordController = TextEditingController();

  @override
  void dispose() {
    _passwordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.status.initialized) {
      return Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Text('Vault not initialized'),
          const SizedBox(height: 16),
          TextField(
            controller: _passwordController,
            obscureText: true,
            decoration: const InputDecoration(labelText: 'Master Password'),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => context.read<VaultBloc>().add(
              InitializeVault(_passwordController.text),
            ),
            child: const Text('Initialize Vault'),
          ),
        ],
      );
    }

    if (!widget.status.unlocked) {
      return Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Text('Vault is locked'),
          const SizedBox(height: 16),
          TextField(
            controller: _passwordController,
            obscureText: true,
            decoration: const InputDecoration(labelText: 'Master Password'),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => context.read<VaultBloc>().add(
              UnlockVault(_passwordController.text),
            ),
            child: const Text('Unlock'),
          ),
        ],
      );
    }

    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        const Icon(Icons.lock_open, size: 48, color: Colors.green),
        const SizedBox(height: 16),
        const Text('Vault is unlocked'),
        const SizedBox(height: 16),
        ElevatedButton(
          onPressed: () =>
              context.read<VaultBloc>().add(const LockVault()),
          child: const Text('Lock'),
        ),
        const SizedBox(height: 8),
        OutlinedButton(
          onPressed: () =>
              context.read<VaultBloc>().add(const LoadPlatforms()),
          child: const Text('View Platforms'),
        ),
      ],
    );
  }
}

class _PlatformsView extends StatelessWidget {
  const _PlatformsView({required this.platforms});
  final List<String> platforms;

  @override
  Widget build(BuildContext context) {
    if (platforms.isEmpty) {
      return const Center(child: Text('No platforms configured'));
    }
    return ListView.builder(
      itemCount: platforms.length,
      itemBuilder: (context, index) => ListTile(
        leading: const Icon(Icons.cloud),
        title: Text(platforms[index]),
      ),
    );
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd frontend && flutter test test/screens/vault_screen_test.dart`
Expected: All 5 tests PASS

- [ ] **Step 5: Commit**

```bash
git add lib/screens/vault_screen.dart test/screens/vault_screen_test.dart
git commit -m "feat(frontend): add VaultScreen with TDD"
```

---

### Task 8: DedupReviewScreen (TDD Widget Test)

**Files:**
- Create: `test/screens/dedup_review_screen_test.dart`
- Create: `lib/screens/dedup_review_screen.dart`

- [ ] **Step 1: Write failing DedupReviewScreen widget tests**

Create `test/screens/dedup_review_screen_test.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory_frontend/blocs/dedup/dedup_bloc.dart';
import 'package:personal_inventory_frontend/models/dedup.dart';
import 'package:personal_inventory_frontend/models/failures.dart';
import 'package:personal_inventory_frontend/models/result.dart';
import 'package:personal_inventory_frontend/repositories/dedup_repository.dart';
import 'package:personal_inventory_frontend/screens/dedup_review_screen.dart';

import '../support/fake_repositories.dart';

Widget _buildTestWidget(DedupRepository repo) {
  return MaterialApp(
    home: BlocProvider<DedupBloc>(
      create: (_) => DedupBloc(repo),
      child: const DedupReviewScreen(),
    ),
  );
}

void main() {
  group('DedupReviewScreen', () {
    testWidgets('shows scan button initially', (tester) async {
      await tester.pumpWidget(_buildTestWidget(FakeDedupRepository()));
      expect(find.text('Scan for Duplicates'), findsOneWidget);
    });

    testWidgets('shows warnings list after scan', (tester) async {
      const warning = DedupWarning(
        id: 'w1', resourceIdA: 'a1', resourceIdB: 'b1',
        similarityScore: 0.92, status: DedupWarningStatus.pending,
      );
      final repo = FakeDedupRepository(
        scanResult: const Success([warning]),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Scan for Duplicates'));
      await tester.pumpAndSettle();

      expect(find.text('92%'), findsOneWidget);
      expect(find.text('Dismiss'), findsOneWidget);
      expect(find.text('Merge'), findsOneWidget);
    });

    testWidgets('shows empty state when no warnings', (tester) async {
      final repo = FakeDedupRepository(
        scanResult: const Success([]),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Scan for Duplicates'));
      await tester.pumpAndSettle();

      expect(find.text('No duplicate warnings found'), findsOneWidget);
    });

    testWidgets('shows error message on failure', (tester) async {
      final repo = FakeDedupRepository(
        scanResult: const Failure(NetworkFailure('offline')),
      );
      await tester.pumpWidget(_buildTestWidget(repo));

      await tester.tap(find.text('Scan for Duplicates'));
      await tester.pumpAndSettle();

      expect(find.textContaining('Error'), findsOneWidget);
    });
  });
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd frontend && flutter test test/screens/dedup_review_screen_test.dart`
Expected: FAIL — `dedup_review_screen.dart` doesn't exist

- [ ] **Step 3: Implement DedupReviewScreen**

Create `lib/screens/dedup_review_screen.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/dedup/dedup_bloc.dart';
import '../models/dedup.dart';

class DedupReviewScreen extends StatelessWidget {
  const DedupReviewScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Dedup Review')),
      body: BlocConsumer<DedupBloc, DedupState>(
        listener: (context, state) {
          if (state is DedupOperationSuccess) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('${state.operationType.name} successful')),
            );
            context.read<DedupBloc>().add(const LoadWarnings());
          }
        },
        builder: (context, state) {
          return Padding(
            padding: const EdgeInsets.all(16),
            child: switch (state) {
              DedupInitial() => _InitialView(),
              DedupLoading() => const Center(child: CircularProgressIndicator()),
              DedupWarningsLoaded(:final warnings) =>
                _WarningsView(warnings: warnings),
              DedupOperationSuccess() => const SizedBox.shrink(),
              DedupError(:final failure) =>
                Center(child: Text('Error: $failure')),
            },
          );
        },
      ),
    );
  }
}

class _InitialView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: ElevatedButton(
        onPressed: () =>
            context.read<DedupBloc>().add(const ScanDuplicates()),
        child: const Text('Scan for Duplicates'),
      ),
    );
  }
}

class _WarningsView extends StatelessWidget {
  const _WarningsView({required this.warnings});
  final List<DedupWarning> warnings;

  @override
  Widget build(BuildContext context) {
    if (warnings.isEmpty) {
      return const Center(child: Text('No duplicate warnings found'));
    }
    return ListView.builder(
      itemCount: warnings.length,
      itemBuilder: (context, index) => _WarningCard(warning: warnings[index]),
    );
  }
}

class _WarningCard extends StatelessWidget {
  const _WarningCard({required this.warning});
  final DedupWarning warning;

  @override
  Widget build(BuildContext context) {
    final percent = (warning.similarityScore * 100).round();
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  '$percent%',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(width: 8),
                const Text('similarity'),
              ],
            ),
            const SizedBox(height: 8),
            Text('Resource A: ${warning.resourceIdA}'),
            Text('Resource B: ${warning.resourceIdB}'),
            const SizedBox(height: 12),
            Row(
              children: [
                OutlinedButton(
                  onPressed: () => context.read<DedupBloc>().add(
                    DismissWarning(warning.id),
                  ),
                  child: const Text('Dismiss'),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  onPressed: () => context.read<DedupBloc>().add(
                    MergeResources(
                      warningId: warning.id,
                      keepId: warning.resourceIdA,
                      discardId: warning.resourceIdB,
                    ),
                  ),
                  child: const Text('Merge'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd frontend && flutter test test/screens/dedup_review_screen_test.dart`
Expected: All 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add lib/screens/dedup_review_screen.dart test/screens/dedup_review_screen_test.dart
git commit -m "feat(frontend): add DedupReviewScreen with TDD"
```

---

### Task 9: EcosystemScreen + Navigation Wiring

**Files:**
- Create: `lib/screens/ecosystem_screen.dart`
- Modify: `lib/main.dart`

- [ ] **Step 1: Create EcosystemScreen hub**

Create `lib/screens/ecosystem_screen.dart`:

```dart
import 'package:flutter/material.dart';

import 'dedup_review_screen.dart';
import 'vault_screen.dart';

class EcosystemScreen extends StatelessWidget {
  const EcosystemScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Ecosystem')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: ListTile(
              leading: const Icon(Icons.lock),
              title: const Text('Credential Vault'),
              subtitle: const Text('Manage platform credentials'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.push(
                context,
                MaterialPageRoute<void>(
                  builder: (_) => const VaultScreen(),
                ),
              ),
            ),
          ),
          Card(
            child: ListTile(
              leading: const Icon(Icons.compare_arrows),
              title: const Text('Dedup Review'),
              subtitle: const Text('Review and resolve duplicates'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.push(
                context,
                MaterialPageRoute<void>(
                  builder: (_) => const DedupReviewScreen(),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
```

- [ ] **Step 2: Wire up main.dart**

Modify `lib/main.dart`:

Add imports at the top:
```dart
import 'blocs/dedup/dedup_bloc.dart';
import 'blocs/vault/vault_bloc.dart';
import 'config/app_config.dart';
import 'repositories/http_dedup_repository.dart';
import 'repositories/http_vault_repository.dart';
import 'screens/ecosystem_screen.dart';
```

Add repositories and bloc providers. Replace the existing `build` method in `PersonalInventoryApp` to add:

1. Create `AppConfig` and HTTP repositories:
```dart
final config = AppConfig.fromEnvironment();
final vaultRepository = HttpVaultRepository(config: config);
final dedupRepository = HttpDedupRepository(config: config);
```

2. Add to MultiBlocProvider's providers list:
```dart
BlocProvider<VaultBloc>(create: (_) => VaultBloc(vaultRepository)),
BlocProvider<DedupBloc>(create: (_) => DedupBloc(dedupRepository)),
```

3. In `_AppShellState`, add the Ecosystem page and navigation destination:
```dart
// In pages list, add:
const EcosystemScreen(),

// In destinations list, add:
NavigationDestination(icon: Icon(Icons.cloud_sync), label: 'Ecosystem'),
```

The final `main.dart` should look like:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'blocs/dedup/dedup_bloc.dart';
import 'blocs/ebook/ebook_bloc.dart';
import 'blocs/game/game_bloc.dart';
import 'blocs/image/image_bloc.dart';
import 'blocs/vault/vault_bloc.dart';
import 'blocs/video/video_bloc.dart';
import 'blocs/web_reader/web_reader_bloc.dart';
import 'config/app_config.dart';
import 'repositories/http_dedup_repository.dart';
import 'repositories/http_vault_repository.dart';
import 'repositories/in_memory_repositories.dart';
import 'screens/bulk_import_screen.dart';
import 'screens/ecosystem_screen.dart';
import 'screens/resource_list_screen.dart';
import 'screens/search_screen.dart';

void main() {
  runApp(const PersonalInventoryApp());
}

class PersonalInventoryApp extends StatelessWidget {
  const PersonalInventoryApp({super.key});

  @override
  Widget build(BuildContext context) {
    final ebookRepository = InMemoryEbookRepository();
    final webReaderRepository = InMemoryWebReaderRepository();
    final imageRepository = InMemoryImageRepository();
    final videoRepository = InMemoryVideoRepository();
    final gameRepository = InMemoryGameRepository();
    final config = AppConfig.fromEnvironment();
    final vaultRepository = HttpVaultRepository(config: config);
    final dedupRepository = HttpDedupRepository(config: config);

    return MultiBlocProvider(
      providers: [
        BlocProvider<EbookBloc>(create: (_) => EbookBloc(ebookRepository)),
        BlocProvider<WebReaderBloc>(create: (_) => WebReaderBloc(webReaderRepository)),
        BlocProvider<ImageBloc>(create: (_) => ImageBloc(imageRepository)),
        BlocProvider<VideoBloc>(create: (_) => VideoBloc(videoRepository)),
        BlocProvider<GameBloc>(create: (_) => GameBloc(gameRepository)),
        BlocProvider<VaultBloc>(create: (_) => VaultBloc(vaultRepository)),
        BlocProvider<DedupBloc>(create: (_) => DedupBloc(dedupRepository)),
      ],
      child: MaterialApp(
        home: _AppShell(ebookRepository: ebookRepository),
      ),
    );
  }
}

class _AppShell extends StatefulWidget {
  const _AppShell({
    required this.ebookRepository,
  });

  final InMemoryEbookRepository ebookRepository;

  @override
  State<_AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<_AppShell> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final pages = [
      const ResourceListScreen(),
      const SearchScreen(),
      BulkImportScreen(ebookRepository: widget.ebookRepository),
      const EcosystemScreen(),
    ];

    return Scaffold(
      body: pages[_index],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        onDestinationSelected: (value) => setState(() => _index = value),
        destinations: const [
          NavigationDestination(icon: Icon(Icons.inventory_2), label: 'Inventory'),
          NavigationDestination(icon: Icon(Icons.search), label: 'Search'),
          NavigationDestination(icon: Icon(Icons.batch_prediction), label: 'Batch'),
          NavigationDestination(icon: Icon(Icons.cloud_sync), label: 'Ecosystem'),
        ],
      ),
    );
  }
}
```

- [ ] **Step 3: Verify build**

Run: `cd frontend && flutter test`
Expected: All tests PASS (existing + new)

- [ ] **Step 4: Commit**

```bash
git add lib/screens/ecosystem_screen.dart lib/main.dart
git commit -m "feat(frontend): add EcosystemScreen hub + navigation wiring"
```

---

### Task 10: Final Verification + CONTEXT.md Update

- [ ] **Step 1: Run full test suite**

Run: `cd frontend && flutter test`
Expected: All tests PASS (existing 110 + ~35 new tests)

- [ ] **Step 2: Verify no analysis issues**

Run: `cd frontend && dart analyze`
Expected: No issues found

- [ ] **Step 3: Update CONTEXT.md**

Append Phase 4 Plan 2 completion summary to `CONTEXT.md`:
- List all new files created
- Summary of vault + dedup frontend features
- Test count (old + new)
- Note deferred items (ecosystem sync, settings)

- [ ] **Step 4: Final commit**

```bash
git add CONTEXT.md
git commit -m "docs: update CONTEXT.md for Phase 4 Plan 2 completion"
```
