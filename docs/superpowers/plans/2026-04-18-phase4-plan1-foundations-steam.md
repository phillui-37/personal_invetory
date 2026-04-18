# Phase 4 Plan 1 — Foundations + Steam + Trait Extraction

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the credential vault, sync engine, dedup warning system, browser session abstraction, and Steam connector — establishing all shared infrastructure and the first working ecosystem integration.

**Architecture:** Hexagonal. New domain types/traits in `domain`, SQLite persistence in `infrastructure`, business logic in `services`, HTTP handlers in `adapters`, runtime wiring in `app`. Steam connector in `plugins`. The `VaultService` in services implements domain's `CredentialVault` trait, combining the `VaultBackend` storage trait (implemented by SQLite) with AES-256-GCM crypto.

**Tech Stack:** Rust (axum, rusqlite, aes-gcm, argon2, strsim, reqwest, serde_json), existing TDD infrastructure (cargo test), chromiumoxide (browser session)

**Spec:** `docs/superpowers/specs/2026-04-18-phase4-ecosystem-integrations-design.md`

---

## File Map

### New Files

| File | Purpose |
|------|---------|
| `backend/domain/src/vault.rs` | CredentialVault + VaultBackend traits, VaultConfig struct |
| `backend/domain/src/sync.rs` | SyncJob, SyncJobStatus, NewSyncJob, SyncJobRepository trait |
| `backend/domain/src/dedup.rs` | DedupWarning, DedupWarningStatus, DedupWarningRepository trait |
| `backend/domain/src/ecosystem.rs` | EcosystemConnector trait, SyncItem, PlatformMetadata (Task 12) |
| `backend/infrastructure/migrations/0013_create_vault_config.sql` | vault_config table |
| `backend/infrastructure/migrations/0014_create_credentials.sql` | credentials table |
| `backend/infrastructure/migrations/0015_create_sync_jobs.sql` | sync_jobs table |
| `backend/infrastructure/migrations/0016_create_dedup_warnings.sql` | dedup_warnings table |
| `backend/infrastructure/src/crypto.rs` | AES-256-GCM encrypt/decrypt + Argon2id key derivation |
| `backend/infrastructure/src/sqlite/vault.rs` | SqliteVaultBackend (implements VaultBackend) |
| `backend/infrastructure/src/sqlite/sync_job.rs` | SqliteSyncJobRepository |
| `backend/infrastructure/src/sqlite/dedup.rs` | SqliteDedupWarningRepository |
| `backend/services/src/vault.rs` | VaultService (implements CredentialVault) |
| `backend/services/src/sync_service.rs` | SyncService (orchestrates connector + repos + dedup) |
| `backend/services/src/dedup.rs` | DedupService (Jaro-Winkler similarity + merge) |
| `backend/adapters/src/vault.rs` | Vault HTTP handlers (unlock, lock, status, credentials CRUD) |
| `backend/adapters/src/ecosystem.rs` | Ecosystem HTTP handlers (sync trigger, history, status) |
| `backend/adapters/src/dedup.rs` | Dedup HTTP handlers (list, dismiss, merge) |
| `backend/plugins/src/browser_session.rs` | BrowserSession abstraction over chromiumoxide |
| `backend/plugins/src/ecosystem/mod.rs` | EcosystemConnector registry + config types |
| `backend/plugins/src/ecosystem/steam.rs` | SteamConnector implementation |
| `backend/app/src/ecosystem_scheduler.rs` | EcosystemScheduler (scheduled platform syncs) |

### Modified Files

| File | Changes |
|------|---------|
| `backend/domain/src/lib.rs` | Add `pub mod vault; pub mod sync; pub mod dedup;` (and `pub mod ecosystem;` in Task 12) |
| `backend/infrastructure/src/sqlite/mod.rs` | Add 4 migrations to SQLITE_MIGRATIONS, 3 new `pub mod` declarations, encode/decode helpers for new enums |
| `backend/infrastructure/src/factory.rs` | Extend AdapterBundle with vault_backend, sync_job_repo, dedup_warning_repo |
| `backend/infrastructure/src/lib.rs` | Add `pub mod crypto;`, update migration count assertion |
| `backend/infrastructure/Cargo.toml` | Add aes-gcm, argon2, rand |
| `backend/plugins/Cargo.toml` | Add reqwest, serde_json |
| `backend/plugins/src/lib.rs` | Add `pub mod browser_session; pub mod ecosystem;`, extend PluginsToml with ecosystem config |
| `backend/services/Cargo.toml` | Add strsim, infrastructure (for crypto) |
| `backend/services/src/lib.rs` | Add `pub mod vault; pub mod sync_service; pub mod dedup;` |
| `backend/adapters/src/lib.rs` | Add `pub mod vault; pub mod ecosystem; pub mod dedup;` |
| `backend/adapters/src/routes.rs` | Add vault, ecosystem, dedup route groups |
| `backend/adapters/src/state.rs` | Add vault_service, sync_service, dedup_service to AppState; extend for_tests() |
| `backend/app/src/runtime.rs` | Wire new services, start ecosystem scheduler |
| `backend/app/Cargo.toml` | (no changes expected — already depends on all workspace crates) |

---

## Task 1: Add Cargo Dependencies

**Files:**
- Modify: `backend/infrastructure/Cargo.toml`
- Modify: `backend/plugins/Cargo.toml`
- Modify: `backend/services/Cargo.toml`

- [ ] **Step 1: Add dependencies to infrastructure/Cargo.toml**

Add under `[dependencies]`:

```toml
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
```

- [ ] **Step 2: Add dependencies to plugins/Cargo.toml**

Add under `[dependencies]`:

```toml
reqwest = { version = "0.12", features = ["json", "cookies"], optional = true }
serde_json = { version = "1", optional = true }
```

Add to `[features]`:

```toml
real-plugins = ["chromiumoxide", "tokio", "futures", "reqwest", "serde_json"]
```

Also add `reqwest` and `serde_json` to `[dev-dependencies]` so tests can use them without feature gates:

```toml
[dev-dependencies]
reqwest = { version = "0.12", features = ["json", "cookies"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 3: Add dependencies to services/Cargo.toml**

Add under `[dependencies]`:

```toml
strsim = "0.11"
infrastructure = { path = "../infrastructure" }
```

- [ ] **Step 4: Verify workspace compiles**

Run: `cd backend && cargo check --workspace 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 5: Commit**

```bash
git add backend/infrastructure/Cargo.toml backend/plugins/Cargo.toml backend/services/Cargo.toml
git commit -m "chore(p4): add aes-gcm, argon2, strsim, reqwest dependencies

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 2: Vault Domain Types and Traits

**Files:**
- Create: `backend/domain/src/vault.rs`
- Modify: `backend/domain/src/lib.rs`

- [ ] **Step 1: Write domain vault types and traits with unit tests**

Create `backend/domain/src/vault.rs`:

```rust
use async_trait::async_trait;
use crate::DomainError;

/// Configuration stored in the vault_config table.
/// Salt for key derivation, plus an encrypted known-plaintext for password verification.
#[derive(Debug, Clone)]
pub struct VaultConfig {
    pub salt: Vec<u8>,
    pub key_check: Vec<u8>,
    pub key_check_nonce: Vec<u8>,
}

/// Low-level storage backend for the credential vault.
/// Handles encrypted blob storage and vault configuration persistence.
/// Implementations must NOT perform any encryption — they store pre-encrypted data.
#[async_trait]
pub trait VaultBackend: Send + Sync {
    async fn get_config(&self) -> Result<Option<VaultConfig>, DomainError>;
    async fn save_config(&self, config: &VaultConfig) -> Result<(), DomainError>;
    async fn store_blob(
        &self,
        platform: &str,
        credential_type: &str,
        encrypted_blob: &[u8],
        nonce: &[u8],
    ) -> Result<(), DomainError>;
    async fn retrieve_blob(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(Vec<u8>, Vec<u8>), DomainError>;
    async fn delete_credential(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(), DomainError>;
    async fn list_platforms(&self) -> Result<Vec<String>, DomainError>;
}

/// High-level credential vault operating on plaintext.
/// Implementations handle encryption/decryption internally.
#[async_trait]
pub trait CredentialVault: Send + Sync {
    async fn initialize(&self, master_password: &str) -> Result<(), DomainError>;
    async fn unlock(&self, master_password: &str) -> Result<(), DomainError>;
    fn lock(&self);
    fn is_unlocked(&self) -> bool;
    async fn is_initialized(&self) -> Result<bool, DomainError>;
    async fn store(
        &self,
        platform: &str,
        credential_type: &str,
        plaintext: &[u8],
    ) -> Result<(), DomainError>;
    async fn retrieve(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<Vec<u8>, DomainError>;
    async fn delete(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(), DomainError>;
    async fn list_platforms(&self) -> Result<Vec<String>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_config_holds_expected_fields() {
        let config = VaultConfig {
            salt: vec![1, 2, 3],
            key_check: vec![4, 5, 6],
            key_check_nonce: vec![7, 8, 9],
        };
        assert_eq!(config.salt, vec![1, 2, 3]);
        assert_eq!(config.key_check, vec![4, 5, 6]);
        assert_eq!(config.key_check_nonce, vec![7, 8, 9]);
    }
}
```

- [ ] **Step 2: Register the vault module in domain/src/lib.rs**

Add after the last `pub mod` statement (or after the existing module declarations):

```rust
pub mod vault;
```

- [ ] **Step 3: Run tests to verify**

Run: `cd backend && cargo test -p domain 2>&1 | tail -10`
Expected: All domain tests pass including `vault_config_holds_expected_fields`

- [ ] **Step 4: Commit**

```bash
git add backend/domain/src/vault.rs backend/domain/src/lib.rs
git commit -m "feat(p4): add credential vault domain types and traits

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 3: Sync Job Domain Types and Traits

**Files:**
- Create: `backend/domain/src/sync.rs`
- Modify: `backend/domain/src/lib.rs`

- [ ] **Step 1: Write sync job domain types with tests**

Create `backend/domain/src/sync.rs`:

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SyncJob {
    pub id: Uuid,
    pub platform: String,
    pub status: SyncJobStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub items_found: u32,
    pub items_created: u32,
    pub items_skipped: u32,
    pub items_failed: u32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct NewSyncJob {
    pub platform: String,
}

#[async_trait]
pub trait SyncJobRepository: Send + Sync {
    async fn create(&self, job: NewSyncJob) -> Result<SyncJob, DomainError>;
    async fn update(&self, job: &SyncJob) -> Result<(), DomainError>;
    async fn get(&self, id: Uuid) -> Result<SyncJob, DomainError>;
    async fn list_by_platform(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_job_status_equality() {
        assert_eq!(SyncJobStatus::Pending, SyncJobStatus::Pending);
        assert_ne!(SyncJobStatus::Pending, SyncJobStatus::Running);
    }

    #[test]
    fn new_sync_job_holds_platform() {
        let job = NewSyncJob {
            platform: "steam".to_string(),
        };
        assert_eq!(job.platform, "steam");
    }
}
```

- [ ] **Step 2: Register sync module in domain/src/lib.rs**

Add:

```rust
pub mod sync;
```

- [ ] **Step 3: Run tests**

Run: `cd backend && cargo test -p domain 2>&1 | tail -10`
Expected: All pass including sync tests

- [ ] **Step 4: Commit**

```bash
git add backend/domain/src/sync.rs backend/domain/src/lib.rs
git commit -m "feat(p4): add sync job domain types and repository trait

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 4: Dedup Warning Domain Types and Traits

**Files:**
- Create: `backend/domain/src/dedup.rs`
- Modify: `backend/domain/src/lib.rs`

- [ ] **Step 1: Write dedup warning domain types with tests**

Create `backend/domain/src/dedup.rs`:

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DedupWarningStatus {
    Pending,
    Dismissed,
    Merged,
}

#[derive(Debug, Clone)]
pub struct DedupWarning {
    pub id: Uuid,
    pub resource_id_a: Uuid,
    pub resource_id_b: Uuid,
    pub similarity_score: f64,
    pub status: DedupWarningStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

pub struct NewDedupWarning {
    pub resource_id_a: Uuid,
    pub resource_id_b: Uuid,
    pub similarity_score: f64,
}

#[async_trait]
pub trait DedupWarningRepository: Send + Sync {
    async fn create(&self, warning: NewDedupWarning) -> Result<DedupWarning, DomainError>;
    async fn list_pending(&self) -> Result<Vec<DedupWarning>, DomainError>;
    async fn dismiss(&self, id: Uuid) -> Result<(), DomainError>;
    async fn mark_merged(&self, id: Uuid) -> Result<(), DomainError>;
    async fn exists_pair(&self, a: Uuid, b: Uuid) -> Result<bool, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_warning_status_equality() {
        assert_eq!(DedupWarningStatus::Pending, DedupWarningStatus::Pending);
        assert_ne!(DedupWarningStatus::Pending, DedupWarningStatus::Dismissed);
    }

    #[test]
    fn new_dedup_warning_holds_fields() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let warning = NewDedupWarning {
            resource_id_a: a,
            resource_id_b: b,
            similarity_score: 0.92,
        };
        assert_eq!(warning.resource_id_a, a);
        assert_eq!(warning.resource_id_b, b);
        assert!((warning.similarity_score - 0.92).abs() < f64::EPSILON);
    }
}
```

- [ ] **Step 2: Register dedup module in domain/src/lib.rs**

Add:

```rust
pub mod dedup;
```

- [ ] **Step 3: Run tests**

Run: `cd backend && cargo test -p domain 2>&1 | tail -10`
Expected: All pass including dedup tests

- [ ] **Step 4: Commit**

```bash
git add backend/domain/src/dedup.rs backend/domain/src/lib.rs
git commit -m "feat(p4): add dedup warning domain types and repository trait

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 5: Crypto Module

**Files:**
- Create: `backend/infrastructure/src/crypto.rs`
- Modify: `backend/infrastructure/src/lib.rs`

- [ ] **Step 1: Write failing crypto tests**

Create `backend/infrastructure/src/crypto.rs`:

```rust
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;
use domain::DomainError;

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], DomainError> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| DomainError::InternalError(format!("key derivation failed: {e}")))?;
    Ok(key)
}

pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), DomainError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| DomainError::InternalError(format!("encryption failed: {e}")))?;
    Ok((ciphertext, nonce_bytes.to_vec()))
}

pub fn decrypt(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, DomainError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| DomainError::InternalError("decryption failed (wrong password?)".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let salt = generate_salt();
        let key = derive_key("test-password", &salt).expect("derive key");
        let plaintext = b"hello world";
        let (ciphertext, nonce) = encrypt(&key, plaintext).expect("encrypt");
        let decrypted = decrypt(&key, &ciphertext, &nonce).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails_decryption() {
        let salt = generate_salt();
        let key1 = derive_key("password-one", &salt).expect("derive key 1");
        let key2 = derive_key("password-two", &salt).expect("derive key 2");
        let (ciphertext, nonce) = encrypt(&key1, b"secret").expect("encrypt");
        let result = decrypt(&key2, &ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn same_password_same_salt_produces_same_key() {
        let salt = generate_salt();
        let key1 = derive_key("same-password", &salt).expect("key 1");
        let key2 = derive_key("same-password", &salt).expect("key 2");
        assert_eq!(key1, key2);
    }

    #[test]
    fn different_salts_produce_different_keys() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let key1 = derive_key("same-password", &salt1).expect("key 1");
        let key2 = derive_key("same-password", &salt2).expect("key 2");
        assert_ne!(key1, key2);
    }

    #[test]
    fn ciphertext_differs_from_plaintext() {
        let salt = generate_salt();
        let key = derive_key("pw", &salt).expect("derive key");
        let plaintext = b"sensitive data";
        let (ciphertext, _) = encrypt(&key, plaintext).expect("encrypt");
        assert_ne!(&ciphertext, plaintext);
    }
}
```

- [ ] **Step 2: Register crypto module in infrastructure/src/lib.rs**

Add:

```rust
pub mod crypto;
```

- [ ] **Step 3: Run tests**

Run: `cd backend && cargo test -p infrastructure -- crypto 2>&1 | tail -15`
Expected: 5 crypto tests pass

- [ ] **Step 4: Commit**

```bash
git add backend/infrastructure/src/crypto.rs backend/infrastructure/src/lib.rs
git commit -m "feat(p4): add AES-256-GCM + Argon2id crypto module

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 6: Vault SQLite Persistence

**Files:**
- Create: `backend/infrastructure/migrations/0013_create_vault_config.sql`
- Create: `backend/infrastructure/migrations/0014_create_credentials.sql`
- Create: `backend/infrastructure/src/sqlite/vault.rs`
- Modify: `backend/infrastructure/src/sqlite/mod.rs`
- Modify: `backend/infrastructure/src/factory.rs`
- Test: `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs`

- [ ] **Step 1: Create migration files**

Create `backend/infrastructure/migrations/0013_create_vault_config.sql`:

```sql
CREATE TABLE IF NOT EXISTS vault_config (
    id TEXT PRIMARY KEY DEFAULT 'default',
    salt BLOB NOT NULL,
    key_check BLOB NOT NULL,
    key_check_nonce BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Create `backend/infrastructure/migrations/0014_create_credentials.sql`:

```sql
CREATE TABLE IF NOT EXISTS credentials (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    credential_type TEXT NOT NULL,
    encrypted_blob BLOB NOT NULL,
    nonce BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(platform, credential_type)
);
```

- [ ] **Step 2: Write the SQLite vault backend**

Create `backend/infrastructure/src/sqlite/vault.rs`:

```rust
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use domain::vault::{VaultBackend, VaultConfig};
use domain::DomainError;

use super::SharedSqliteConnection;
use super::map_sqlite_error;

pub struct SqliteVaultBackend {
    conn: SharedSqliteConnection,
}

impl SqliteVaultBackend {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl VaultBackend for SqliteVaultBackend {
    async fn get_config(&self) -> Result<Option<VaultConfig>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT salt, key_check, key_check_nonce FROM vault_config WHERE id = 'default'")
            .map_err(map_sqlite_error)?;
        let result = stmt.query_row([], |row| {
            Ok(VaultConfig {
                salt: row.get(0)?,
                key_check: row.get(1)?,
                key_check_nonce: row.get(2)?,
            })
        });
        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(map_sqlite_error(e)),
        }
    }

    async fn save_config(&self, config: &VaultConfig) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO vault_config (id, salt, key_check, key_check_nonce) VALUES ('default', ?1, ?2, ?3)",
            rusqlite::params![config.salt, config.key_check, config.key_check_nonce],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn store_blob(
        &self,
        platform: &str,
        credential_type: &str,
        encrypted_blob: &[u8],
        nonce: &[u8],
    ) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO credentials (id, platform, credential_type, encrypted_blob, nonce)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(platform, credential_type)
             DO UPDATE SET encrypted_blob = ?4, nonce = ?5, updated_at = datetime('now')",
            rusqlite::params![id, platform, credential_type, encrypted_blob, nonce],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn retrieve_blob(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(Vec<u8>, Vec<u8>), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT encrypted_blob, nonce FROM credentials WHERE platform = ?1 AND credential_type = ?2",
            rusqlite::params![platform, credential_type],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DomainError::NotFound(format!("no credential for {platform}/{credential_type}"))
            }
            other => map_sqlite_error(other),
        })
    }

    async fn delete_credential(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM credentials WHERE platform = ?1 AND credential_type = ?2",
            rusqlite::params![platform, credential_type],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn list_platforms(&self) -> Result<Vec<String>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT DISTINCT platform FROM credentials ORDER BY platform")
            .map_err(map_sqlite_error)?;
        let platforms = stmt
            .query_map([], |row| row.get(0))
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<String>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(platforms)
    }
}
```

- [ ] **Step 3: Register vault module and migrations in sqlite/mod.rs**

Add to module declarations:

```rust
pub mod vault;
```

Update `SQLITE_MIGRATIONS` array to include migrations 0013 and 0014 (add 2 entries to the existing 12):

```rust
pub const SQLITE_MIGRATIONS: [&str; 14] = [
    // ... existing 12 entries ...
    include_str!("../../migrations/0013_create_vault_config.sql"),
    include_str!("../../migrations/0014_create_credentials.sql"),
];
```

- [ ] **Step 4: Extend AdapterBundle in factory.rs**

Add new field to `AdapterBundle`:

```rust
pub vault_backend: Arc<dyn domain::vault::VaultBackend>,
```

And in `AdapterFactory::from_url()`, after the existing repo creations, add:

```rust
let vault_backend = Arc::new(sqlite::vault::SqliteVaultBackend::new(conn.clone()));
```

Add `vault_backend` to the returned AdapterBundle.

- [ ] **Step 5: Update migration count assertion in infrastructure/src/lib.rs**

If there's an assertion checking the number of migrations, update it from 12 to 14.

- [ ] **Step 6: Write integration test**

Add to `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs`:

```rust
#[test]
fn sqlite_vault_backend_config_roundtrip() {
    let bundle = sqlite_bundle();
    block_on(async {
        let config = bundle.vault_backend.get_config().await.expect("get config");
        assert!(config.is_none(), "no config initially");

        let vault_config = domain::vault::VaultConfig {
            salt: vec![1, 2, 3, 4],
            key_check: vec![5, 6, 7, 8],
            key_check_nonce: vec![9, 10, 11],
        };
        bundle.vault_backend.save_config(&vault_config).await.expect("save config");

        let loaded = bundle.vault_backend.get_config().await.expect("get config").expect("some");
        assert_eq!(loaded.salt, vec![1, 2, 3, 4]);
        assert_eq!(loaded.key_check, vec![5, 6, 7, 8]);
        assert_eq!(loaded.key_check_nonce, vec![9, 10, 11]);
    });
}

#[test]
fn sqlite_vault_backend_blob_store_retrieve_delete() {
    let bundle = sqlite_bundle();
    block_on(async {
        bundle.vault_backend
            .store_blob("steam", "api_key", b"encrypted-data", b"nonce-12byte")
            .await
            .expect("store");

        let (blob, nonce) = bundle.vault_backend
            .retrieve_blob("steam", "api_key")
            .await
            .expect("retrieve");
        assert_eq!(blob, b"encrypted-data");
        assert_eq!(nonce, b"nonce-12byte");

        let platforms = bundle.vault_backend.list_platforms().await.expect("list");
        assert_eq!(platforms, vec!["steam".to_string()]);

        bundle.vault_backend.delete_credential("steam", "api_key").await.expect("delete");
        let result = bundle.vault_backend.retrieve_blob("steam", "api_key").await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    });
}

#[test]
fn sqlite_vault_backend_upsert_overwrites() {
    let bundle = sqlite_bundle();
    block_on(async {
        bundle.vault_backend
            .store_blob("dlsite", "cookie_jar", b"old", b"nonce1-12byte")
            .await
            .expect("store 1");
        bundle.vault_backend
            .store_blob("dlsite", "cookie_jar", b"new", b"nonce2-12byte")
            .await
            .expect("store 2");

        let (blob, nonce) = bundle.vault_backend
            .retrieve_blob("dlsite", "cookie_jar")
            .await
            .expect("retrieve");
        assert_eq!(blob, b"new");
        assert_eq!(nonce, b"nonce2-12byte");
    });
}
```

Make sure the test file's `sqlite_bundle()` helper also returns the new `vault_backend` field — update it to access `bundle.vault_backend`.

- [ ] **Step 7: Run tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -20`
Expected: All tests pass including 3 new vault backend tests

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(p4): add vault SQLite persistence with migrations and integration tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 7: Sync Job and Dedup Warning SQLite Persistence

**Files:**
- Create: `backend/infrastructure/migrations/0015_create_sync_jobs.sql`
- Create: `backend/infrastructure/migrations/0016_create_dedup_warnings.sql`
- Create: `backend/infrastructure/src/sqlite/sync_job.rs`
- Create: `backend/infrastructure/src/sqlite/dedup.rs`
- Modify: `backend/infrastructure/src/sqlite/mod.rs`
- Modify: `backend/infrastructure/src/factory.rs`
- Test: `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs`

- [ ] **Step 1: Create migration files**

Create `backend/infrastructure/migrations/0015_create_sync_jobs.sql`:

```sql
CREATE TABLE IF NOT EXISTS sync_jobs (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TEXT,
    completed_at TEXT,
    items_found INTEGER NOT NULL DEFAULT 0,
    items_created INTEGER NOT NULL DEFAULT 0,
    items_skipped INTEGER NOT NULL DEFAULT 0,
    items_failed INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Create `backend/infrastructure/migrations/0016_create_dedup_warnings.sql`:

```sql
CREATE TABLE IF NOT EXISTS dedup_warnings (
    id TEXT PRIMARY KEY,
    resource_id_a TEXT NOT NULL REFERENCES resources(id),
    resource_id_b TEXT NOT NULL REFERENCES resources(id),
    similarity_score REAL NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT,
    UNIQUE(resource_id_a, resource_id_b)
);
```

- [ ] **Step 2: Write SqliteSyncJobRepository**

Create `backend/infrastructure/src/sqlite/sync_job.rs`:

```rust
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use domain::sync::{NewSyncJob, SyncJob, SyncJobRepository, SyncJobStatus};
use domain::DomainError;

use super::{map_sqlite_error, parse_timestamp, SharedSqliteConnection};

pub struct SqliteSyncJobRepository {
    conn: SharedSqliteConnection,
}

impl SqliteSyncJobRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

fn encode_sync_status(status: &SyncJobStatus) -> &'static str {
    match status {
        SyncJobStatus::Pending => "pending",
        SyncJobStatus::Running => "running",
        SyncJobStatus::Completed => "completed",
        SyncJobStatus::Failed => "failed",
    }
}

fn decode_sync_status(s: &str) -> SyncJobStatus {
    match s {
        "running" => SyncJobStatus::Running,
        "completed" => SyncJobStatus::Completed,
        "failed" => SyncJobStatus::Failed,
        _ => SyncJobStatus::Pending,
    }
}

#[async_trait]
impl SyncJobRepository for SqliteSyncJobRepository {
    async fn create(&self, job: NewSyncJob) -> Result<SyncJob, DomainError> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4();
        let now = Utc::now();
        let id_str = id.to_string();
        let created_at_str = now.to_rfc3339();
        conn.execute(
            "INSERT INTO sync_jobs (id, platform, status, created_at) VALUES (?1, ?2, 'pending', ?3)",
            rusqlite::params![id_str, job.platform, created_at_str],
        )
        .map_err(map_sqlite_error)?;
        Ok(SyncJob {
            id,
            platform: job.platform,
            status: SyncJobStatus::Pending,
            started_at: None,
            completed_at: None,
            items_found: 0,
            items_created: 0,
            items_skipped: 0,
            items_failed: 0,
            error_message: None,
            created_at: now,
        })
    }

    async fn update(&self, job: &SyncJob) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let id_str = job.id.to_string();
        let status_str = encode_sync_status(&job.status);
        let started_at = job.started_at.map(|t| t.to_rfc3339());
        let completed_at = job.completed_at.map(|t| t.to_rfc3339());
        conn.execute(
            "UPDATE sync_jobs SET status = ?1, started_at = ?2, completed_at = ?3,
             items_found = ?4, items_created = ?5, items_skipped = ?6, items_failed = ?7,
             error_message = ?8 WHERE id = ?9",
            rusqlite::params![
                status_str, started_at, completed_at,
                job.items_found, job.items_created, job.items_skipped, job.items_failed,
                job.error_message, id_str
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<SyncJob, DomainError> {
        let conn = self.conn.lock().unwrap();
        let id_str = id.to_string();
        conn.query_row(
            "SELECT platform, status, started_at, completed_at, items_found, items_created,
             items_skipped, items_failed, error_message, created_at FROM sync_jobs WHERE id = ?1",
            rusqlite::params![id_str],
            |row| {
                let status_str: String = row.get(1)?;
                let started_at: Option<String> = row.get(2)?;
                let completed_at: Option<String> = row.get(3)?;
                let created_at_str: String = row.get(9)?;
                Ok(SyncJob {
                    id,
                    platform: row.get(0)?,
                    status: decode_sync_status(&status_str),
                    started_at: started_at.as_deref().map(parse_timestamp),
                    completed_at: completed_at.as_deref().map(parse_timestamp),
                    items_found: u32::try_from(row.get::<_, i64>(4)?).unwrap_or(0),
                    items_created: u32::try_from(row.get::<_, i64>(5)?).unwrap_or(0),
                    items_skipped: u32::try_from(row.get::<_, i64>(6)?).unwrap_or(0),
                    items_failed: u32::try_from(row.get::<_, i64>(7)?).unwrap_or(0),
                    error_message: row.get(8)?,
                    created_at: parse_timestamp(&created_at_str),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DomainError::NotFound(format!("sync job {id} not found"))
            }
            other => map_sqlite_error(other),
        })
    }

    async fn list_by_platform(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, status, started_at, completed_at, items_found, items_created,
                 items_skipped, items_failed, error_message, created_at
                 FROM sync_jobs WHERE platform = ?1 ORDER BY created_at DESC",
            )
            .map_err(map_sqlite_error)?;
        let jobs = stmt
            .query_map(rusqlite::params![platform], |row| {
                let id_str: String = row.get(0)?;
                let status_str: String = row.get(1)?;
                let started_at: Option<String> = row.get(2)?;
                let completed_at: Option<String> = row.get(3)?;
                let created_at_str: String = row.get(9)?;
                Ok(SyncJob {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    platform: platform.to_string(),
                    status: decode_sync_status(&status_str),
                    started_at: started_at.as_deref().map(parse_timestamp),
                    completed_at: completed_at.as_deref().map(parse_timestamp),
                    items_found: u32::try_from(row.get::<_, i64>(4)?).unwrap_or(0),
                    items_created: u32::try_from(row.get::<_, i64>(5)?).unwrap_or(0),
                    items_skipped: u32::try_from(row.get::<_, i64>(6)?).unwrap_or(0),
                    items_failed: u32::try_from(row.get::<_, i64>(7)?).unwrap_or(0),
                    error_message: row.get(8)?,
                    created_at: parse_timestamp(&created_at_str),
                })
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(jobs)
    }
}
```

- [ ] **Step 3: Write SqliteDedupWarningRepository**

Create `backend/infrastructure/src/sqlite/dedup.rs`:

```rust
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use domain::dedup::{DedupWarning, DedupWarningRepository, DedupWarningStatus, NewDedupWarning};
use domain::DomainError;

use super::{map_sqlite_error, parse_timestamp, SharedSqliteConnection};

pub struct SqliteDedupWarningRepository {
    conn: SharedSqliteConnection,
}

impl SqliteDedupWarningRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

fn decode_dedup_status(s: &str) -> DedupWarningStatus {
    match s {
        "dismissed" => DedupWarningStatus::Dismissed,
        "merged" => DedupWarningStatus::Merged,
        _ => DedupWarningStatus::Pending,
    }
}

#[async_trait]
impl DedupWarningRepository for SqliteDedupWarningRepository {
    async fn create(&self, warning: NewDedupWarning) -> Result<DedupWarning, DomainError> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4();
        let now = Utc::now();
        let id_str = id.to_string();
        let a_str = warning.resource_id_a.to_string();
        let b_str = warning.resource_id_b.to_string();
        let created_at_str = now.to_rfc3339();
        conn.execute(
            "INSERT INTO dedup_warnings (id, resource_id_a, resource_id_b, similarity_score, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
            rusqlite::params![id_str, a_str, b_str, warning.similarity_score, created_at_str],
        )
        .map_err(map_sqlite_error)?;
        Ok(DedupWarning {
            id,
            resource_id_a: warning.resource_id_a,
            resource_id_b: warning.resource_id_b,
            similarity_score: warning.similarity_score,
            status: DedupWarningStatus::Pending,
            created_at: now,
            resolved_at: None,
        })
    }

    async fn list_pending(&self) -> Result<Vec<DedupWarning>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, resource_id_a, resource_id_b, similarity_score, created_at
                 FROM dedup_warnings WHERE status = 'pending' ORDER BY similarity_score DESC",
            )
            .map_err(map_sqlite_error)?;
        let warnings = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let a_str: String = row.get(1)?;
                let b_str: String = row.get(2)?;
                let created_at_str: String = row.get(4)?;
                Ok(DedupWarning {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    resource_id_a: Uuid::parse_str(&a_str).unwrap_or_default(),
                    resource_id_b: Uuid::parse_str(&b_str).unwrap_or_default(),
                    similarity_score: row.get(3)?,
                    status: DedupWarningStatus::Pending,
                    created_at: parse_timestamp(&created_at_str),
                    resolved_at: None,
                })
            })
            .map_err(map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_error)?;
        Ok(warnings)
    }

    async fn dismiss(&self, id: Uuid) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE dedup_warnings SET status = 'dismissed', resolved_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id.to_string()],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn mark_merged(&self, id: Uuid) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE dedup_warnings SET status = 'merged', resolved_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id.to_string()],
        )
        .map_err(map_sqlite_error)?;
        Ok(())
    }

    async fn exists_pair(&self, a: Uuid, b: Uuid) -> Result<bool, DomainError> {
        let conn = self.conn.lock().unwrap();
        let a_str = a.to_string();
        let b_str = b.to_string();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dedup_warnings
                 WHERE (resource_id_a = ?1 AND resource_id_b = ?2)
                    OR (resource_id_a = ?2 AND resource_id_b = ?1)",
                rusqlite::params![a_str, b_str],
                |row| row.get(0),
            )
            .map_err(map_sqlite_error)?;
        Ok(count > 0)
    }
}
```

- [ ] **Step 4: Register modules and migrations in sqlite/mod.rs**

Add module declarations:

```rust
pub mod sync_job;
pub mod dedup;
```

Update `SQLITE_MIGRATIONS` to 16 entries:

```rust
pub const SQLITE_MIGRATIONS: [&str; 16] = [
    // ... existing 14 entries ...
    include_str!("../../migrations/0015_create_sync_jobs.sql"),
    include_str!("../../migrations/0016_create_dedup_warnings.sql"),
];
```

- [ ] **Step 5: Extend AdapterBundle in factory.rs**

Add fields:

```rust
pub sync_job_repo: Arc<dyn domain::sync::SyncJobRepository>,
pub dedup_warning_repo: Arc<dyn domain::dedup::DedupWarningRepository>,
```

In `from_url()`:

```rust
let sync_job_repo = Arc::new(sqlite::sync_job::SqliteSyncJobRepository::new(conn.clone()));
let dedup_warning_repo = Arc::new(sqlite::dedup::SqliteDedupWarningRepository::new(conn.clone()));
```

- [ ] **Step 6: Write integration tests**

Add to `backend/infrastructure/tests/sqlite_repository_adapters_tdd.rs`:

```rust
#[test]
fn sqlite_sync_job_create_update_and_list() {
    let bundle = sqlite_bundle();
    block_on(async {
        use domain::sync::{NewSyncJob, SyncJobStatus};

        let job = bundle.sync_job_repo
            .create(NewSyncJob { platform: "steam".to_string() })
            .await
            .expect("create");
        assert_eq!(job.platform, "steam");
        assert_eq!(job.status, SyncJobStatus::Pending);
        assert_eq!(job.items_found, 0);

        let mut updated = job.clone();
        updated.status = SyncJobStatus::Completed;
        updated.items_found = 10;
        updated.items_created = 8;
        updated.items_skipped = 2;
        bundle.sync_job_repo.update(&updated).await.expect("update");

        let loaded = bundle.sync_job_repo.get(updated.id).await.expect("get");
        assert_eq!(loaded.status, SyncJobStatus::Completed);
        assert_eq!(loaded.items_found, 10);
        assert_eq!(loaded.items_created, 8);
        assert_eq!(loaded.items_skipped, 2);

        let list = bundle.sync_job_repo.list_by_platform("steam").await.expect("list");
        assert_eq!(list.len(), 1);
    });
}

#[test]
fn sqlite_dedup_warning_create_dismiss_and_exists() {
    let bundle = sqlite_bundle();
    block_on(async {
        use domain::dedup::NewDedupWarning;

        let res_a = bundle.resource_repo
            .create(NewResource { title: "Zelda TOTK".to_string(), notes: None, resource_type: ResourceType::Game })
            .await.expect("create a");
        let res_b = bundle.resource_repo
            .create(NewResource { title: "Zelda Tears".to_string(), notes: None, resource_type: ResourceType::Game })
            .await.expect("create b");

        let warning = bundle.dedup_warning_repo
            .create(NewDedupWarning {
                resource_id_a: res_a.id,
                resource_id_b: res_b.id,
                similarity_score: 0.91,
            })
            .await.expect("create warning");

        let pending = bundle.dedup_warning_repo.list_pending().await.expect("list");
        assert_eq!(pending.len(), 1);
        assert!((pending[0].similarity_score - 0.91).abs() < f64::EPSILON);

        let exists = bundle.dedup_warning_repo.exists_pair(res_a.id, res_b.id).await.expect("exists");
        assert!(exists);
        let exists_reverse = bundle.dedup_warning_repo.exists_pair(res_b.id, res_a.id).await.expect("exists reverse");
        assert!(exists_reverse);

        bundle.dedup_warning_repo.dismiss(warning.id).await.expect("dismiss");
        let pending_after = bundle.dedup_warning_repo.list_pending().await.expect("list after");
        assert_eq!(pending_after.len(), 0);
    });
}
```

- [ ] **Step 7: Run tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -20`
Expected: All tests pass including new sync_job and dedup_warning tests

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(p4): add sync job and dedup warning SQLite persistence

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 8: Vault Service

**Files:**
- Create: `backend/services/src/vault.rs`
- Modify: `backend/services/src/lib.rs`
- Modify: `backend/services/Cargo.toml`
- Test: `backend/services/tests/services_tdd.rs`

- [ ] **Step 1: Write VaultService**

Create `backend/services/src/vault.rs`:

```rust
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use domain::vault::{CredentialVault, VaultBackend, VaultConfig};
use domain::DomainError;
use infrastructure::crypto;

pub struct VaultService {
    backend: Arc<dyn VaultBackend>,
    key: Arc<Mutex<Option<[u8; 32]>>>,
}

impl VaultService {
    pub fn new(backend: Arc<dyn VaultBackend>) -> Self {
        Self {
            backend,
            key: Arc::new(Mutex::new(None)),
        }
    }

    fn get_key(&self) -> Result<[u8; 32], DomainError> {
        self.key
            .lock()
            .unwrap()
            .ok_or_else(|| DomainError::ValidationError("vault is locked".to_string()))
    }
}

#[async_trait]
impl CredentialVault for VaultService {
    async fn initialize(&self, master_password: &str) -> Result<(), DomainError> {
        let existing = self.backend.get_config().await?;
        if existing.is_some() {
            return Err(DomainError::Conflict("vault already initialized".to_string()));
        }

        let salt = crypto::generate_salt();
        let key = crypto::derive_key(master_password, &salt)?;
        let (key_check, key_check_nonce) = crypto::encrypt(&key, b"vault-ok")?;

        let config = VaultConfig {
            salt: salt.to_vec(),
            key_check,
            key_check_nonce,
        };
        self.backend.save_config(&config).await?;
        *self.key.lock().unwrap() = Some(key);
        Ok(())
    }

    async fn unlock(&self, master_password: &str) -> Result<(), DomainError> {
        let config = self.backend.get_config().await?.ok_or_else(|| {
            DomainError::NotFound("vault not initialized — call initialize first".to_string())
        })?;

        let key = crypto::derive_key(master_password, &config.salt)?;
        let plaintext = crypto::decrypt(&key, &config.key_check, &config.key_check_nonce)?;
        if plaintext != b"vault-ok" {
            return Err(DomainError::ValidationError("wrong master password".to_string()));
        }

        *self.key.lock().unwrap() = Some(key);
        Ok(())
    }

    fn lock(&self) {
        *self.key.lock().unwrap() = None;
    }

    fn is_unlocked(&self) -> bool {
        self.key.lock().unwrap().is_some()
    }

    async fn is_initialized(&self) -> Result<bool, DomainError> {
        Ok(self.backend.get_config().await?.is_some())
    }

    async fn store(
        &self,
        platform: &str,
        credential_type: &str,
        plaintext: &[u8],
    ) -> Result<(), DomainError> {
        let key = self.get_key()?;
        let (encrypted, nonce) = crypto::encrypt(&key, plaintext)?;
        self.backend
            .store_blob(platform, credential_type, &encrypted, &nonce)
            .await
    }

    async fn retrieve(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<Vec<u8>, DomainError> {
        let key = self.get_key()?;
        let (encrypted, nonce) = self.backend.retrieve_blob(platform, credential_type).await?;
        crypto::decrypt(&key, &encrypted, &nonce)
    }

    async fn delete(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(), DomainError> {
        self.backend.delete_credential(platform, credential_type).await
    }

    async fn list_platforms(&self) -> Result<Vec<String>, DomainError> {
        self.backend.list_platforms().await
    }
}
```

- [ ] **Step 2: Register module in services/src/lib.rs**

Add:

```rust
pub mod vault;
```

- [ ] **Step 3: Write VaultService tests**

Add to `backend/services/tests/services_tdd.rs`:

```rust
use domain::vault::CredentialVault;
use services::vault::VaultService;

// Helper: creates a VaultService backed by a real SQLite vault backend.
// Reuse the sqlite_bundle approach from infrastructure tests.
fn vault_service() -> VaultService {
    use infrastructure::sqlite::open_sqlite_connection;
    use infrastructure::sqlite::vault::SqliteVaultBackend;
    use std::sync::Arc;

    let conn = open_sqlite_connection(":memory:").expect("open");
    let backend = Arc::new(SqliteVaultBackend::new(conn));
    VaultService::new(backend)
}

#[tokio::test]
async fn vault_initialize_and_unlock_roundtrip() {
    let vault = vault_service();
    assert!(!vault.is_initialized().await.unwrap());
    assert!(!vault.is_unlocked());

    vault.initialize("my-master-pw").await.unwrap();
    assert!(vault.is_initialized().await.unwrap());
    assert!(vault.is_unlocked());

    vault.lock();
    assert!(!vault.is_unlocked());

    vault.unlock("my-master-pw").await.unwrap();
    assert!(vault.is_unlocked());
}

#[tokio::test]
async fn vault_wrong_password_fails_unlock() {
    let vault = vault_service();
    vault.initialize("correct-pw").await.unwrap();
    vault.lock();
    let result = vault.unlock("wrong-pw").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn vault_store_retrieve_credential_roundtrip() {
    let vault = vault_service();
    vault.initialize("pw").await.unwrap();

    vault.store("steam", "api_key", b"my-steam-key").await.unwrap();
    let retrieved = vault.retrieve("steam", "api_key").await.unwrap();
    assert_eq!(retrieved, b"my-steam-key");
}

#[tokio::test]
async fn vault_locked_store_fails() {
    let vault = vault_service();
    vault.initialize("pw").await.unwrap();
    vault.lock();

    let result = vault.store("steam", "api_key", b"key").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn vault_double_initialize_fails() {
    let vault = vault_service();
    vault.initialize("pw1").await.unwrap();
    let result = vault.initialize("pw2").await;
    assert!(matches!(result, Err(domain::DomainError::Conflict(_))));
}
```

- [ ] **Step 4: Run tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -20`
Expected: All pass including 5 new vault service tests

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(p4): add VaultService with encrypt/decrypt and master password lifecycle

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 9: Dedup Service

**Files:**
- Create: `backend/services/src/dedup.rs`
- Modify: `backend/services/src/lib.rs`
- Test: `backend/services/tests/services_tdd.rs`

- [ ] **Step 1: Write DedupService**

Create `backend/services/src/dedup.rs`:

```rust
use std::sync::Arc;

use domain::dedup::{DedupWarningRepository, NewDedupWarning, DedupWarning};
use domain::{DomainError, Resource, ResourceRepository, LocationRepository};
use strsim::jaro_winkler;
use uuid::Uuid;

pub struct DedupService {
    warning_repo: Arc<dyn DedupWarningRepository>,
    resource_repo: Arc<dyn ResourceRepository>,
    location_repo: Arc<dyn LocationRepository>,
    threshold: f64,
}

impl DedupService {
    pub fn new(
        warning_repo: Arc<dyn DedupWarningRepository>,
        resource_repo: Arc<dyn ResourceRepository>,
        location_repo: Arc<dyn LocationRepository>,
        threshold: f64,
    ) -> Self {
        Self {
            warning_repo,
            resource_repo,
            location_repo,
            threshold,
        }
    }

    pub async fn check_new_resources(&self, new_ids: &[Uuid]) -> Result<Vec<DedupWarning>, DomainError> {
        let all_resources = self.resource_repo.list().await?;
        let mut warnings = Vec::new();

        for &new_id in new_ids {
            let new_res = match all_resources.iter().find(|r| r.id == new_id) {
                Some(r) => r,
                None => continue,
            };
            let norm_new = normalize_title(&new_res.title);

            for existing in &all_resources {
                if existing.id == new_id {
                    continue;
                }
                let norm_existing = normalize_title(&existing.title);
                let score = jaro_winkler(&norm_new, &norm_existing);
                if score >= self.threshold {
                    let already_exists = self
                        .warning_repo
                        .exists_pair(new_id, existing.id)
                        .await?;
                    if !already_exists {
                        let warning = self
                            .warning_repo
                            .create(NewDedupWarning {
                                resource_id_a: new_id,
                                resource_id_b: existing.id,
                                similarity_score: score,
                            })
                            .await?;
                        warnings.push(warning);
                    }
                }
            }
        }
        Ok(warnings)
    }

    pub async fn list_pending(&self) -> Result<Vec<DedupWarning>, DomainError> {
        self.warning_repo.list_pending().await
    }

    pub async fn dismiss(&self, warning_id: Uuid) -> Result<(), DomainError> {
        self.warning_repo.dismiss(warning_id).await
    }

    pub async fn merge(&self, warning_id: Uuid) -> Result<(), DomainError> {
        let warnings = self.warning_repo.list_pending().await?;
        let warning = warnings
            .into_iter()
            .find(|w| w.id == warning_id)
            .ok_or_else(|| DomainError::NotFound(format!("warning {warning_id} not found")))?;

        // Copy locations from B to A
        let locations_b = self.location_repo.list(warning.resource_id_b).await?;
        for loc in locations_b {
            // Ignore conflicts (duplicate locations)
            let _ = self
                .location_repo
                .add(
                    warning.resource_id_a,
                    domain::NewLocation {
                        device_id: loc.device_id,
                        path_or_url: loc.path_or_url,
                        storage_type: loc.storage_type,
                    },
                )
                .await;
        }

        // Merge notes
        let res_a = self.resource_repo.get_by_id(warning.resource_id_a).await?;
        let res_b = self.resource_repo.get_by_id(warning.resource_id_b).await?;
        if let Some(notes_b) = &res_b.notes {
            let merged_notes = match &res_a.notes {
                Some(notes_a) => format!("{notes_a}\n---\n{notes_b}"),
                None => notes_b.clone(),
            };
            self.resource_repo
                .update(domain::UpdateResource {
                    id: warning.resource_id_a,
                    title: None,
                    notes: Some(merged_notes),
                })
                .await?;
        }

        // Delete resource B
        self.resource_repo.delete(warning.resource_id_b).await?;

        // Mark warning as merged
        self.warning_repo.mark_merged(warning_id).await?;
        Ok(())
    }
}

fn normalize_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let stripped: String = lower
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    let trimmed = stripped.trim();
    // Strip common prefixes
    for prefix in &["the ", "a ", "an "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest.to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_title_strips_prefix_and_punctuation() {
        assert_eq!(normalize_title("The Legend of Zelda"), "legend of zelda");
        assert_eq!(normalize_title("A Tale: Of Two Cities!"), "tale of two cities");
        assert_eq!(normalize_title("  Hello World  "), "hello world");
    }

    #[test]
    fn jaro_winkler_similar_titles() {
        let a = normalize_title("Zelda: Tears of the Kingdom");
        let b = normalize_title("The Legend of Zelda: Tears of the Kingdom");
        let score = jaro_winkler(&a, &b);
        assert!(score > 0.8, "score was {score}");
    }
}
```

- [ ] **Step 2: Register module**

Add to `services/src/lib.rs`:

```rust
pub mod dedup;
```

- [ ] **Step 3: Run tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -15`
Expected: All pass including normalize_title and jaro_winkler tests

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(p4): add DedupService with Jaro-Winkler similarity and merge logic

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 10: Vault, Ecosystem, and Dedup HTTP Handlers

**Files:**
- Create: `backend/adapters/src/vault.rs`
- Create: `backend/adapters/src/ecosystem.rs`
- Create: `backend/adapters/src/dedup.rs`
- Modify: `backend/adapters/src/lib.rs`
- Modify: `backend/adapters/src/routes.rs`
- Modify: `backend/adapters/src/state.rs`

- [ ] **Step 1: Write vault handlers**

Create `backend/adapters/src/vault.rs`:

```rust
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use domain::vault::CredentialVault;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct UnlockRequest {
    pub master_password: String,
}

#[derive(Serialize)]
pub struct VaultStatusResponse {
    pub unlocked: bool,
    pub initialized: bool,
}

#[derive(Deserialize)]
pub struct StoreCredentialRequest {
    pub platform: String,
    pub credential_type: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct PlatformListResponse {
    pub platforms: Vec<String>,
}

pub async fn unlock(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UnlockRequest>,
) -> impl IntoResponse {
    let vault = match &state.vault_service {
        Some(v) => v,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "vault not configured").into_response(),
    };

    let initialized = vault.is_initialized().await.unwrap_or(false);
    let result = if initialized {
        vault.unlock(&req.master_password).await
    } else {
        vault.initialize(&req.master_password).await
    };

    match result {
        Ok(()) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

pub async fn lock(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(vault) = &state.vault_service {
        vault.lock();
    }
    StatusCode::OK
}

pub async fn status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let vault = match &state.vault_service {
        Some(v) => v,
        None => {
            return Json(VaultStatusResponse {
                unlocked: false,
                initialized: false,
            })
            .into_response()
        }
    };
    let initialized = vault.is_initialized().await.unwrap_or(false);
    let unlocked = vault.is_unlocked();
    Json(VaultStatusResponse {
        unlocked,
        initialized,
    })
    .into_response()
}

pub async fn store_credential(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StoreCredentialRequest>,
) -> impl IntoResponse {
    let vault = match &state.vault_service {
        Some(v) => v,
        None => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    match vault
        .store(&req.platform, &req.credential_type, req.value.as_bytes())
        .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(domain::DomainError::ValidationError(_)) => StatusCode::FORBIDDEN.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn list_credentials(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let vault = match &state.vault_service {
        Some(v) => v,
        None => return Json(PlatformListResponse { platforms: vec![] }).into_response(),
    };
    match vault.list_platforms().await {
        Ok(platforms) => Json(PlatformListResponse { platforms }).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
```

- [ ] **Step 2: Write ecosystem handlers**

Create `backend/adapters/src/ecosystem.rs`:

```rust
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

use crate::state::AppState;

#[derive(Serialize)]
pub struct SyncTriggerResponse {
    pub job_id: String,
}

#[derive(Serialize)]
pub struct SyncJobResponse {
    pub id: String,
    pub platform: String,
    pub status: String,
    pub items_found: u32,
    pub items_created: u32,
    pub items_skipped: u32,
    pub items_failed: u32,
    pub error_message: Option<String>,
}

pub async fn trigger_sync(
    State(state): State<Arc<AppState>>,
    Path(platform): Path<String>,
) -> impl IntoResponse {
    let sync_service = match &state.sync_service {
        Some(s) => s,
        None => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    match sync_service.trigger_sync(&platform).await {
        Ok(job) => Json(SyncTriggerResponse {
            job_id: job.id.to_string(),
        })
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn sync_history(
    State(state): State<Arc<AppState>>,
    Path(platform): Path<String>,
) -> impl IntoResponse {
    let sync_service = match &state.sync_service {
        Some(s) => s,
        None => return Json(Vec::<SyncJobResponse>::new()).into_response(),
    };
    match sync_service.list_jobs(&platform).await {
        Ok(jobs) => Json(
            jobs.into_iter()
                .map(|j| SyncJobResponse {
                    id: j.id.to_string(),
                    platform: j.platform,
                    status: format!("{:?}", j.status),
                    items_found: j.items_found,
                    items_created: j.items_created,
                    items_skipped: j.items_skipped,
                    items_failed: j.items_failed,
                    error_message: j.error_message,
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn ecosystem_status(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // Returns a summary of all platforms; details filled in Task 11 (runtime wiring)
    Json(serde_json::json!({ "platforms": [] }))
}
```

- [ ] **Step 3: Write dedup handlers**

Create `backend/adapters/src/dedup.rs`:

```rust
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Serialize)]
pub struct DedupWarningResponse {
    pub id: String,
    pub resource_id_a: String,
    pub resource_id_b: String,
    pub similarity_score: f64,
}

pub async fn list_warnings(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let dedup = match &state.dedup_service {
        Some(d) => d,
        None => return Json(Vec::<DedupWarningResponse>::new()).into_response(),
    };
    match dedup.list_pending().await {
        Ok(warnings) => Json(
            warnings
                .into_iter()
                .map(|w| DedupWarningResponse {
                    id: w.id.to_string(),
                    resource_id_a: w.resource_id_a.to_string(),
                    resource_id_b: w.resource_id_b.to_string(),
                    similarity_score: w.similarity_score,
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn dismiss_warning(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dedup = match &state.dedup_service {
        Some(d) => d,
        None => return StatusCode::SERVICE_UNAVAILABLE,
    };
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    match dedup.dismiss(uuid).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn merge_warning(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dedup = match &state.dedup_service {
        Some(d) => d,
        None => return StatusCode::SERVICE_UNAVAILABLE,
    };
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    match dedup.merge(uuid).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
```

- [ ] **Step 4: Register modules in adapters/src/lib.rs**

Add:

```rust
pub mod vault;
pub mod ecosystem;
pub mod dedup;
```

- [ ] **Step 5: Extend AppState in adapters/src/state.rs**

Add fields to `AppState`:

```rust
pub vault_service: Option<Arc<dyn domain::vault::CredentialVault>>,
pub sync_service: Option<Arc<services::sync_service::SyncService>>,
pub dedup_service: Option<Arc<services::dedup::DedupService>>,
```

Update `AppState::new()` to accept these (or set them to `None` initially, with builder methods).

Update `AppState::for_tests()` to set all three to `None`.

- [ ] **Step 6: Add routes in adapters/src/routes.rs**

Add new route groups inside `build_router()`, after existing routes:

```rust
// Vault routes
.route("/api/v1/vault/unlock", post(vault::unlock))
.route("/api/v1/vault/lock", post(vault::lock))
.route("/api/v1/vault/status", get(vault::status))
.route("/api/v1/vault/credentials", post(vault::store_credential))
.route("/api/v1/vault/credentials", get(vault::list_credentials))
// Ecosystem routes
.route("/api/v1/ecosystem/:platform/sync", post(ecosystem::trigger_sync))
.route("/api/v1/ecosystem/:platform/syncs", get(ecosystem::sync_history))
.route("/api/v1/ecosystem/status", get(ecosystem::ecosystem_status))
// Dedup routes
.route("/api/v1/dedup-warnings", get(dedup::list_warnings))
.route("/api/v1/dedup-warnings/:id/dismiss", post(dedup::dismiss_warning))
.route("/api/v1/dedup-warnings/:id/merge", post(dedup::merge_warning))
```

All vault/ecosystem/dedup routes need the `auth_middleware` layer.

- [ ] **Step 7: Write handler integration tests**

Add to `backend/adapters/tests/handlers_tdd.rs`:

```rust
#[tokio::test]
async fn vault_status_returns_uninitialized() {
    let app = app_with_openapi("{}");
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/vault/status")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.expect("bytes");
    let result: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(result["initialized"], false);
    assert_eq!(result["unlocked"], false);
}

#[tokio::test]
async fn dedup_warnings_list_returns_empty() {
    let app = app_with_openapi("{}");
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dedup-warnings")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn ecosystem_status_returns_ok() {
    let app = app_with_openapi("{}");
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/ecosystem/status")
                .header("authorization", "Bearer secret-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
}
```

- [ ] **Step 8: Run tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -20`
Expected: All pass including 3 new handler tests

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat(p4): add vault, ecosystem, and dedup HTTP handlers and routes

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 11: Browser Session Abstraction

**Files:**
- Create: `backend/plugins/src/browser_session.rs`
- Modify: `backend/plugins/src/lib.rs`

- [ ] **Step 1: Write BrowserSession**

Create `backend/plugins/src/browser_session.rs`:

```rust
use domain::DomainError;

/// A serializable cookie representation for vault storage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

/// Abstraction over a headless browser page.
/// Used by ecosystem connectors for login flows.
/// Real implementation uses chromiumoxide; tests can mock this trait.
#[async_trait::async_trait]
pub trait BrowserPage: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<(), DomainError>;
    async fn wait_for_selector(&self, css: &str, timeout_ms: u64) -> Result<(), DomainError>;
    async fn extract_text(&self, css: &str) -> Result<String, DomainError>;
    async fn extract_all_text(&self, css: &str) -> Result<Vec<String>, DomainError>;
    async fn extract_html(&self, css: &str) -> Result<String, DomainError>;
    async fn click(&self, css: &str) -> Result<(), DomainError>;
    async fn fill(&self, css: &str, value: &str) -> Result<(), DomainError>;
    async fn get_cookies(&self) -> Result<Vec<Cookie>, DomainError>;
    async fn set_cookies(&self, cookies: Vec<Cookie>) -> Result<(), DomainError>;
}

/// No-op browser page for testing.
pub struct NoopBrowserPage;

#[async_trait::async_trait]
impl BrowserPage for NoopBrowserPage {
    async fn navigate(&self, _url: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn wait_for_selector(&self, _css: &str, _timeout_ms: u64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn extract_text(&self, _css: &str) -> Result<String, DomainError> {
        Ok(String::new())
    }
    async fn extract_all_text(&self, _css: &str) -> Result<Vec<String>, DomainError> {
        Ok(vec![])
    }
    async fn extract_html(&self, _css: &str) -> Result<String, DomainError> {
        Ok(String::new())
    }
    async fn click(&self, _css: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn fill(&self, _css: &str, _value: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_cookies(&self) -> Result<Vec<Cookie>, DomainError> {
        Ok(vec![])
    }
    async fn set_cookies(&self, _cookies: Vec<Cookie>) -> Result<(), DomainError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_serializes_to_json() {
        let cookie = Cookie {
            name: "session".to_string(),
            value: "abc123".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
        };
        let json = serde_json::to_string(&cookie).expect("serialize");
        assert!(json.contains("session"));
        assert!(json.contains("abc123"));
    }

    #[tokio::test]
    async fn noop_browser_page_returns_ok() {
        let page = NoopBrowserPage;
        page.navigate("https://example.com").await.unwrap();
        let text = page.extract_text(".title").await.unwrap();
        assert!(text.is_empty());
    }
}
```

- [ ] **Step 2: Register module in plugins/src/lib.rs**

Add:

```rust
pub mod browser_session;
```

- [ ] **Step 3: Run tests**

Run: `cd backend && cargo test -p plugins 2>&1 | tail -15`
Expected: All plugin tests pass including browser_session tests

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(p4): add BrowserSession trait and NoopBrowserPage for ecosystem connectors

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 12: Steam Connector

**Files:**
- Create: `backend/plugins/src/ecosystem/mod.rs`
- Create: `backend/plugins/src/ecosystem/steam.rs`
- Modify: `backend/plugins/src/lib.rs`
- Create: `backend/services/src/sync_service.rs`
- Modify: `backend/services/src/lib.rs`

- [ ] **Step 1: Write ecosystem module with SyncItem and connector config**

Create `backend/plugins/src/ecosystem/mod.rs`:

```rust
pub mod steam;

use domain::{DomainError, NewGameMeta, NewEbookMeta, NewImageMeta, NewVideoMeta, ResourceType, StorageType};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct NewLocation {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: StorageType,
}

#[derive(Debug, Clone)]
pub enum PlatformMetadata {
    Game(NewGameMeta),
    Ebook(NewEbookMeta),
    Image(NewImageMeta),
    Video(NewVideoMeta),
    None,
}

#[derive(Debug, Clone)]
pub struct SyncItem {
    pub external_id: String,
    pub title: String,
    pub resource_type: ResourceType,
    pub notes: Option<String>,
    pub metadata: PlatformMetadata,
    pub location: Option<NewLocation>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EcosystemConfig {
    #[serde(default)]
    pub steam: Option<SteamConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SteamConfig {
    pub enabled: bool,
    #[serde(default = "default_sync_interval")]
    pub sync_interval_secs: u64,
    pub steam_api_key: Option<String>,
    pub steam_id: Option<String>,
    pub device_id: Option<String>,
}

fn default_sync_interval() -> u64 {
    86400
}
```

- [ ] **Step 2: Write SteamConnector**

Create `backend/plugins/src/ecosystem/steam.rs`:

```rust
use domain::DomainError;
use domain::{NewGameMeta, ResourceType, StorageType};
use serde::Deserialize;

use super::{NewLocation, PlatformMetadata, SyncItem};

pub struct SteamConnector {
    api_key: String,
    steam_id: String,
    device_id: String,
    base_url: String,
}

impl SteamConnector {
    pub fn new(api_key: String, steam_id: String, device_id: String) -> Self {
        Self {
            api_key,
            steam_id,
            device_id,
            base_url: "https://api.steampowered.com".to_string(),
        }
    }

    /// For testing: override the Steam API base URL.
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub fn platform(&self) -> &str {
        "steam"
    }

    pub async fn sync_library(&self) -> Result<Vec<SyncItem>, DomainError> {
        let url = format!(
            "{}/IPlayerService/GetOwnedGames/v1?key={}&steamid={}&include_appinfo=true&format=json",
            self.base_url, self.api_key, self.steam_id
        );
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| DomainError::InternalError(format!("steam API error: {e}")))?;
        let body: SteamOwnedGamesResponse = resp
            .json()
            .await
            .map_err(|e| DomainError::InternalError(format!("steam parse error: {e}")))?;

        let games = body.response.games.unwrap_or_default();
        Ok(games.into_iter().map(|g| self.map_game(g)).collect())
    }

    fn map_game(&self, game: SteamGame) -> SyncItem {
        let playtime_hours = game.playtime_forever / 60;
        let notes = if playtime_hours > 0 {
            Some(format!("Steam playtime: {}h", playtime_hours))
        } else {
            None
        };

        SyncItem {
            external_id: game.appid.to_string(),
            title: game.name.unwrap_or_else(|| format!("Steam App {}", game.appid)),
            resource_type: ResourceType::Game,
            notes,
            metadata: PlatformMetadata::Game(NewGameMeta {
                platform: Some("PC".to_string()),
                store: Some("Steam".to_string()),
                developer: None,
                publisher: None,
                manual_notes: None,
            }),
            location: Some(NewLocation {
                device_id: self.device_id.clone(),
                path_or_url: format!("steam://rungameid/{}", game.appid),
                storage_type: StorageType::Platform,
            }),
        }
    }

    /// Parse a pre-fetched JSON response (for unit testing without HTTP).
    pub fn parse_owned_games(json: &str) -> Result<Vec<SteamGame>, DomainError> {
        let resp: SteamOwnedGamesResponse = serde_json::from_str(json)
            .map_err(|e| DomainError::InternalError(format!("parse error: {e}")))?;
        Ok(resp.response.games.unwrap_or_default())
    }
}

#[derive(Debug, Deserialize)]
struct SteamOwnedGamesResponse {
    response: SteamOwnedGamesBody,
}

#[derive(Debug, Deserialize)]
struct SteamOwnedGamesBody {
    games: Option<Vec<SteamGame>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SteamGame {
    pub appid: u64,
    pub name: Option<String>,
    #[serde(default)]
    pub playtime_forever: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &str = r#"{
        "response": {
            "game_count": 2,
            "games": [
                {"appid": 440, "name": "Team Fortress 2", "playtime_forever": 1200},
                {"appid": 570, "name": "Dota 2", "playtime_forever": 0}
            ]
        }
    }"#;

    #[test]
    fn parse_steam_owned_games_response() {
        let games = SteamConnector::parse_owned_games(SAMPLE_RESPONSE).expect("parse");
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].appid, 440);
        assert_eq!(games[0].name.as_deref(), Some("Team Fortress 2"));
        assert_eq!(games[0].playtime_forever, 1200);
        assert_eq!(games[1].appid, 570);
    }

    #[test]
    fn map_game_creates_correct_sync_item() {
        let connector = SteamConnector::new(
            "key".to_string(),
            "id".to_string(),
            "my-pc".to_string(),
        );
        let game = SteamGame {
            appid: 440,
            name: Some("Team Fortress 2".to_string()),
            playtime_forever: 1200,
        };
        let item = connector.map_game(game);
        assert_eq!(item.title, "Team Fortress 2");
        assert_eq!(item.resource_type, ResourceType::Game);
        assert_eq!(item.notes, Some("Steam playtime: 20h".to_string()));
        assert_eq!(item.external_id, "440");
        match &item.metadata {
            PlatformMetadata::Game(meta) => {
                assert_eq!(meta.platform, Some("PC".to_string()));
                assert_eq!(meta.store, Some("Steam".to_string()));
            }
            _ => panic!("expected Game metadata"),
        }
        let loc = item.location.as_ref().expect("location");
        assert_eq!(loc.device_id, "my-pc");
        assert_eq!(loc.path_or_url, "steam://rungameid/440");
    }

    #[test]
    fn map_game_with_zero_playtime_has_no_notes() {
        let connector = SteamConnector::new("k".into(), "i".into(), "pc".into());
        let game = SteamGame {
            appid: 570,
            name: Some("Dota 2".to_string()),
            playtime_forever: 0,
        };
        let item = connector.map_game(game);
        assert!(item.notes.is_none());
    }

    #[test]
    fn empty_games_list_parses_ok() {
        let json = r#"{"response": {"game_count": 0}}"#;
        let games = SteamConnector::parse_owned_games(json).expect("parse");
        assert!(games.is_empty());
    }
}
```

- [ ] **Step 3: Register ecosystem module in plugins/src/lib.rs**

Add:

```rust
pub mod ecosystem;
```

- [ ] **Step 4: Write SyncService**

Create `backend/services/src/sync_service.rs`:

```rust
use std::sync::Arc;

use domain::sync::{NewSyncJob, SyncJob, SyncJobRepository, SyncJobStatus};
use domain::{DomainError, NewResource, ResourceRepository, ResourceType, LocationRepository};
use chrono::Utc;
use uuid::Uuid;

use plugins::ecosystem::{PlatformMetadata, SyncItem};

pub struct SyncService {
    sync_job_repo: Arc<dyn SyncJobRepository>,
    resource_repo: Arc<dyn ResourceRepository>,
    location_repo: Arc<dyn LocationRepository>,
}

impl SyncService {
    pub fn new(
        sync_job_repo: Arc<dyn SyncJobRepository>,
        resource_repo: Arc<dyn ResourceRepository>,
        location_repo: Arc<dyn LocationRepository>,
    ) -> Self {
        Self {
            sync_job_repo,
            resource_repo,
            location_repo,
        }
    }

    pub async fn trigger_sync(&self, platform: &str) -> Result<SyncJob, DomainError> {
        let job = self
            .sync_job_repo
            .create(NewSyncJob {
                platform: platform.to_string(),
            })
            .await?;
        Ok(job)
    }

    pub async fn execute_sync(
        &self,
        job_id: Uuid,
        items: Vec<SyncItem>,
    ) -> Result<SyncJob, DomainError> {
        let mut job = self.sync_job_repo.get(job_id).await?;
        job.status = SyncJobStatus::Running;
        job.started_at = Some(Utc::now());
        job.items_found = u32::try_from(items.len()).unwrap_or(u32::MAX);
        self.sync_job_repo.update(&job).await?;

        let mut created_ids = Vec::new();

        for item in items {
            match self.create_resource_from_sync_item(item).await {
                Ok(resource_id) => {
                    job.items_created += 1;
                    created_ids.push(resource_id);
                }
                Err(DomainError::Conflict(_)) => {
                    job.items_skipped += 1;
                }
                Err(_) => {
                    job.items_failed += 1;
                }
            }
        }

        job.status = SyncJobStatus::Completed;
        job.completed_at = Some(Utc::now());
        self.sync_job_repo.update(&job).await?;
        Ok(job)
    }

    async fn create_resource_from_sync_item(
        &self,
        item: SyncItem,
    ) -> Result<Uuid, DomainError> {
        let resource = self
            .resource_repo
            .create(NewResource {
                title: item.title,
                notes: item.notes,
                resource_type: item.resource_type,
            })
            .await?;

        // TODO: upsert type-specific meta when meta repos are injected
        // For now, sync only creates the base resource + location

        if let Some(loc) = item.location {
            let _ = self
                .location_repo
                .add(
                    resource.id,
                    domain::NewLocation {
                        device_id: loc.device_id,
                        path_or_url: loc.path_or_url,
                        storage_type: loc.storage_type,
                    },
                )
                .await;
        }

        Ok(resource.id)
    }

    pub async fn list_jobs(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError> {
        self.sync_job_repo.list_by_platform(platform).await
    }

    pub fn created_resource_ids_placeholder(&self) -> Vec<Uuid> {
        // Used by caller to pass to DedupService::check_new_resources after sync completes
        vec![]
    }
}
```

- [ ] **Step 5: Register sync_service module in services/src/lib.rs**

Add:

```rust
pub mod sync_service;
```

- [ ] **Step 6: Run tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -20`
Expected: All pass including Steam connector tests (4 unit tests)

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(p4): add SteamConnector with library parsing and SyncService

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 13: Runtime Wiring and Ecosystem Scheduler

**Files:**
- Create: `backend/app/src/ecosystem_scheduler.rs`
- Modify: `backend/app/src/runtime.rs`
- Modify: `backend/plugins/src/lib.rs` (extend PluginsToml)

- [ ] **Step 1: Extend PluginsToml with ecosystem config**

In `backend/plugins/src/lib.rs`, add to the `PluginsToml` struct:

```rust
#[derive(Debug, Deserialize, Default)]
pub struct PluginsToml {
    pub web_checker: Option<WebCheckerConfig>,
    pub ecosystem: Option<ecosystem::EcosystemConfig>,
}
```

- [ ] **Step 2: Write EcosystemScheduler**

Create `backend/app/src/ecosystem_scheduler.rs`:

```rust
use std::sync::Arc;
use std::time::Duration;

use domain::sync::SyncJobRepository;
use domain::vault::CredentialVault;
use plugins::ecosystem::EcosystemConfig;
use plugins::ecosystem::steam::SteamConnector;
use services::sync_service::SyncService;
use services::dedup::DedupService;
use tokio_util::sync::CancellationToken;

pub struct EcosystemScheduler {
    config: EcosystemConfig,
    sync_service: Arc<SyncService>,
    dedup_service: Arc<DedupService>,
    vault: Arc<dyn CredentialVault>,
    shutdown: CancellationToken,
}

impl EcosystemScheduler {
    pub fn new(
        config: EcosystemConfig,
        sync_service: Arc<SyncService>,
        dedup_service: Arc<DedupService>,
        vault: Arc<dyn CredentialVault>,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            config,
            sync_service,
            dedup_service,
            vault,
            shutdown,
        }
    }

    pub fn start(self) {
        tokio::spawn(async move {
            self.run_loop().await;
        });
    }

    async fn run_loop(&self) {
        let interval = self
            .config
            .steam
            .as_ref()
            .map(|s| Duration::from_secs(s.sync_interval_secs))
            .unwrap_or(Duration::from_secs(86400));

        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {
                    self.tick().await;
                }
                _ = self.shutdown.cancelled() => {
                    break;
                }
            }
        }
    }

    async fn tick(&self) {
        if !self.vault.is_unlocked() {
            return;
        }

        if let Some(steam_config) = &self.config.steam {
            if steam_config.enabled {
                if let (Some(api_key), Some(steam_id)) =
                    (&steam_config.steam_api_key, &steam_config.steam_id)
                {
                    let device_id = steam_config
                        .device_id
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    let connector =
                        SteamConnector::new(api_key.clone(), steam_id.clone(), device_id);
                    match connector.sync_library().await {
                        Ok(items) => {
                            if let Ok(job) = self.sync_service.trigger_sync("steam").await {
                                let _ = self.sync_service.execute_sync(job.id, items).await;
                            }
                        }
                        Err(e) => {
                            eprintln!("steam sync error: {e:?}");
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 3: Wire new services in runtime.rs**

In `build_app_router()`, after existing service wiring:

1. Create VaultService:
```rust
let vault_backend = Arc::new(infrastructure::sqlite::vault::SqliteVaultBackend::new(conn.clone()));
let vault_service: Arc<dyn domain::vault::CredentialVault> = Arc::new(services::vault::VaultService::new(vault_backend));
```

2. Create SyncService:
```rust
let sync_service = Arc::new(services::sync_service::SyncService::new(
    bundle.sync_job_repo.clone(),
    bundle.resource_repo.clone(),
    bundle.location_repo.clone(),
));
```

3. Create DedupService:
```rust
let dedup_service = Arc::new(services::dedup::DedupService::new(
    bundle.dedup_warning_repo.clone(),
    bundle.resource_repo.clone(),
    bundle.location_repo.clone(),
    0.85, // default threshold
));
```

4. Add to AppState:
```rust
vault_service: Some(vault_service.clone()),
sync_service: Some(sync_service.clone()),
dedup_service: Some(dedup_service.clone()),
```

5. Start EcosystemScheduler if config has ecosystem section:
```rust
if let Some(ecosystem_config) = plugins_toml.ecosystem {
    let eco_scheduler = EcosystemScheduler::new(
        ecosystem_config,
        sync_service.clone(),
        dedup_service.clone(),
        vault_service.clone(),
        shutdown_token.clone(),
    );
    eco_scheduler.start();
}
```

- [ ] **Step 4: Register ecosystem_scheduler module in app**

In `backend/app/src/main.rs` or wherever modules are declared, add:

```rust
mod ecosystem_scheduler;
```

- [ ] **Step 5: Write runtime integration test**

Add to `backend/app/src/runtime.rs` tests:

```rust
#[tokio::test]
async fn app_router_wires_vault_service() {
    let app = build_test_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/vault/status")
                .header("authorization", "Bearer test-key")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
}
```

- [ ] **Step 6: Run full test suite**

Run: `cd backend && cargo test --workspace 2>&1 | tail -20`
Expected: All tests pass

Run: `cd frontend && flutter test 2>&1 | tail -5`
Expected: All 110 frontend tests pass (no frontend changes)

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(p4): wire vault, sync, dedup services and ecosystem scheduler into runtime

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 14: EcosystemConnector Trait Extraction

**Files:**
- Create: `backend/domain/src/ecosystem.rs`
- Modify: `backend/domain/src/lib.rs`
- Modify: `backend/plugins/src/ecosystem/mod.rs`
- Modify: `backend/plugins/src/ecosystem/steam.rs`

- [ ] **Step 1: Create EcosystemConnector trait in domain**

Create `backend/domain/src/ecosystem.rs`:

```rust
use async_trait::async_trait;
use crate::DomainError;
use crate::vault::CredentialVault;

/// Trait for ecosystem platform connectors (Steam, DLSite, FANZA, BookWalker, Kindle).
/// Each platform implements this trait to provide library sync and metadata fetching.
#[async_trait]
pub trait EcosystemConnector: Send + Sync {
    fn platform(&self) -> &str;
    async fn authenticate(&self, vault: &dyn CredentialVault) -> Result<(), DomainError>;
    async fn is_authenticated(&self) -> bool;
    async fn sync_library(&self) -> Result<Vec<crate::ecosystem::SyncItem>, DomainError>;
}
```

Wait — `SyncItem` is currently in `plugins::ecosystem`. The trait should reference domain types. Move `SyncItem` and `PlatformMetadata` into domain.

- [ ] **Step 1 (revised): Move SyncItem and PlatformMetadata to domain**

Create `backend/domain/src/ecosystem.rs`:

```rust
use async_trait::async_trait;
use crate::{
    DomainError, NewEbookMeta, NewGameMeta, NewImageMeta, NewVideoMeta,
    ResourceType, StorageType,
};
use crate::vault::CredentialVault;

#[derive(Debug, Clone)]
pub struct SyncItemLocation {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: StorageType,
}

#[derive(Debug, Clone)]
pub enum PlatformMetadata {
    Game(NewGameMeta),
    Ebook(NewEbookMeta),
    Image(NewImageMeta),
    Video(NewVideoMeta),
    None,
}

#[derive(Debug, Clone)]
pub struct SyncItem {
    pub external_id: String,
    pub title: String,
    pub resource_type: ResourceType,
    pub notes: Option<String>,
    pub metadata: PlatformMetadata,
    pub location: Option<SyncItemLocation>,
}

#[async_trait]
pub trait EcosystemConnector: Send + Sync {
    fn platform(&self) -> &str;
    async fn authenticate(&self, vault: &dyn CredentialVault) -> Result<(), DomainError>;
    async fn is_authenticated(&self) -> bool;
    async fn sync_library(&self) -> Result<Vec<SyncItem>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_item_can_be_constructed() {
        let item = SyncItem {
            external_id: "440".to_string(),
            title: "TF2".to_string(),
            resource_type: ResourceType::Game,
            notes: None,
            metadata: PlatformMetadata::None,
            location: None,
        };
        assert_eq!(item.external_id, "440");
    }
}
```

- [ ] **Step 2: Register ecosystem module in domain/src/lib.rs**

Add:

```rust
pub mod ecosystem;
```

- [ ] **Step 3: Refactor plugins/ecosystem to use domain types**

Update `backend/plugins/src/ecosystem/mod.rs` to re-export from domain instead of defining its own types:

```rust
pub mod steam;

pub use domain::ecosystem::{SyncItem, SyncItemLocation, PlatformMetadata};

// Keep EcosystemConfig here since it's plugin configuration, not domain
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EcosystemConfig {
    #[serde(default)]
    pub steam: Option<SteamConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SteamConfig {
    pub enabled: bool,
    #[serde(default = "default_sync_interval")]
    pub sync_interval_secs: u64,
    pub steam_api_key: Option<String>,
    pub steam_id: Option<String>,
    pub device_id: Option<String>,
}

fn default_sync_interval() -> u64 {
    86400
}
```

- [ ] **Step 4: Update SteamConnector to implement EcosystemConnector trait**

Update `backend/plugins/src/ecosystem/steam.rs` — change `SteamConnector` to implement the domain trait:

```rust
use async_trait::async_trait;
use domain::ecosystem::{EcosystemConnector, SyncItem, SyncItemLocation, PlatformMetadata};
use domain::vault::CredentialVault;
use domain::DomainError;
use domain::{NewGameMeta, ResourceType, StorageType};
// ... keep existing SteamConnector struct and parse logic ...

#[async_trait]
impl EcosystemConnector for SteamConnector {
    fn platform(&self) -> &str {
        "steam"
    }

    async fn authenticate(&self, _vault: &dyn CredentialVault) -> Result<(), DomainError> {
        // Steam uses API key, no interactive auth needed
        Ok(())
    }

    async fn is_authenticated(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn sync_library(&self) -> Result<Vec<SyncItem>, DomainError> {
        // existing sync_library implementation
        // ... (keep the reqwest-based implementation)
    }
}
```

Update the `map_game` method to use `SyncItemLocation` instead of the old `NewLocation` type.

- [ ] **Step 5: Update SyncService to use domain::ecosystem::SyncItem**

In `backend/services/src/sync_service.rs`, change imports from `plugins::ecosystem::SyncItem` to `domain::ecosystem::SyncItem`. Update `create_resource_from_sync_item` to use `SyncItemLocation`.

- [ ] **Step 6: Run full test suite**

Run: `cd backend && cargo test --workspace 2>&1 | tail -20`
Expected: All tests pass (refactor preserved behavior)

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(p4): extract EcosystemConnector trait into domain, Steam implements it

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Task 15: Final Verification and Context Update

**Files:**
- Modify: `CONTEXT.md`

- [ ] **Step 1: Run full backend test suite**

Run: `cd backend && cargo test --workspace 2>&1`
Expected: All tests pass. Note the total count.

- [ ] **Step 2: Run full frontend test suite**

Run: `cd frontend && flutter test 2>&1 | tail -5`
Expected: All 110 tests pass (no frontend changes in Plan 1)

- [ ] **Step 3: Update CONTEXT.md**

Append Phase 4 Plan 1 completion summary to `CONTEXT.md`:

- Credential vault (AES-256-GCM + Argon2id, SQLite backend, VaultService)
- Sync engine (SyncJob, SyncJobRepository, SyncService)
- Dedup warning system (Jaro-Winkler similarity, merge workflow)
- Browser session abstraction (BrowserPage trait + NoopBrowserPage)
- Steam connector (API-based library sync, implements EcosystemConnector trait)
- EcosystemScheduler (scheduled sync via plugins.toml config)
- New API routes: /vault/*, /ecosystem/*, /dedup-warnings/*
- EcosystemConnector trait extracted into domain for Plan 2 connectors
- 4 new SQLite migrations (0013-0016)
- New dependencies: aes-gcm, argon2, strsim, reqwest

- [ ] **Step 4: Commit**

```bash
git add CONTEXT.md
git commit -m "docs(p4): update CONTEXT.md with Phase 4 Plan 1 completion summary

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```
