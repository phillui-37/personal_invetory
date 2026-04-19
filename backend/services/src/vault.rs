use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use domain::vault::{CredentialVault, VaultBackend, VaultConfig};
use domain::DomainError;
use infrastructure::crypto;

pub struct VaultService {
    backend: Arc<dyn VaultBackend>,
    derived_key: Arc<Mutex<Option<[u8; 32]>>>,
}

impl VaultService {
    pub fn new(backend: Arc<dyn VaultBackend>) -> Self {
        Self {
            backend,
            derived_key: Arc::new(Mutex::new(None)),
        }
    }

    fn get_key(&self) -> Result<[u8; 32], DomainError> {
        self.derived_key
            .lock()
            .map_err(|_| DomainError::InternalError("lock poisoned".to_string()))?
            .ok_or_else(|| DomainError::ValidationError("vault is locked".to_string()))
    }
}

#[async_trait]
impl CredentialVault for VaultService {
    async fn initialize(&self, master_password: &str) -> Result<(), DomainError> {
        if self.is_initialized().await? {
            return Err(DomainError::ValidationError(
                "vault already initialized".to_string(),
            ));
        }
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(master_password, &salt)?;
        let check_plaintext = b"vault-check";
        let (key_check, key_check_nonce) = crypto::encrypt(&key, check_plaintext)?;
        let config = VaultConfig {
            salt: salt.to_vec(),
            key_check,
            key_check_nonce,
        };
        self.backend.save_config(&config).await?;
        *self
            .derived_key
            .lock()
            .map_err(|_| DomainError::InternalError("lock poisoned".to_string()))? = Some(key);
        Ok(())
    }

    async fn unlock(&self, master_password: &str) -> Result<(), DomainError> {
        let config = self.backend.get_config().await?.ok_or_else(|| {
            DomainError::ValidationError("vault not initialized".to_string())
        })?;
        let key = crypto::derive_key(master_password, &config.salt)?;
        crypto::decrypt(&key, &config.key_check, &config.key_check_nonce)?;
        *self
            .derived_key
            .lock()
            .map_err(|_| DomainError::InternalError("lock poisoned".to_string()))? = Some(key);
        Ok(())
    }

    fn lock(&self) {
        // Recover from poison so the key is always cleared even after a panic.
        let mut guard = match self.derived_key.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        *guard = None;
    }

    fn is_unlocked(&self) -> bool {
        self.derived_key
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
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
        let (ciphertext, nonce) = crypto::encrypt(&key, plaintext)?;
        self.backend
            .store_blob(platform, credential_type, &ciphertext, &nonce)
            .await
    }

    async fn retrieve(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<Vec<u8>, DomainError> {
        let key = self.get_key()?;
        let (ciphertext, nonce) = self.backend.retrieve_blob(platform, credential_type).await?;
        crypto::decrypt(&key, &ciphertext, &nonce)
    }

    async fn delete(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(), DomainError> {
        self.backend
            .delete_credential(platform, credential_type)
            .await
    }

    async fn list_platforms(&self) -> Result<Vec<String>, DomainError> {
        self.backend.list_platforms().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::collections::HashMap;

    struct InMemoryVaultBackend {
        config: Mutex<Option<VaultConfig>>,
        blobs: Mutex<HashMap<(String, String), (Vec<u8>, Vec<u8>)>>,
    }

    impl InMemoryVaultBackend {
        fn new() -> Self {
            Self {
                config: Mutex::new(None),
                blobs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl VaultBackend for InMemoryVaultBackend {
        async fn get_config(&self) -> Result<Option<VaultConfig>, DomainError> {
            Ok(self.config.lock().unwrap().clone())
        }
        async fn save_config(&self, config: &VaultConfig) -> Result<(), DomainError> {
            *self.config.lock().unwrap() = Some(config.clone());
            Ok(())
        }
        async fn store_blob(
            &self,
            platform: &str,
            cred_type: &str,
            blob: &[u8],
            nonce: &[u8],
        ) -> Result<(), DomainError> {
            self.blobs.lock().unwrap().insert(
                (platform.to_string(), cred_type.to_string()),
                (blob.to_vec(), nonce.to_vec()),
            );
            Ok(())
        }
        async fn retrieve_blob(
            &self,
            platform: &str,
            cred_type: &str,
        ) -> Result<(Vec<u8>, Vec<u8>), DomainError> {
            self.blobs
                .lock()
                .unwrap()
                .get(&(platform.to_string(), cred_type.to_string()))
                .cloned()
                .ok_or_else(|| DomainError::NotFound("credential not found".to_string()))
        }
        async fn delete_credential(
            &self,
            platform: &str,
            cred_type: &str,
        ) -> Result<(), DomainError> {
            self.blobs
                .lock()
                .unwrap()
                .remove(&(platform.to_string(), cred_type.to_string()));
            Ok(())
        }
        async fn list_platforms(&self) -> Result<Vec<String>, DomainError> {
            let mut platforms: Vec<String> = self
                .blobs
                .lock()
                .unwrap()
                .keys()
                .map(|(p, _)| p.clone())
                .collect();
            platforms.sort();
            platforms.dedup();
            Ok(platforms)
        }
    }

    fn make_service() -> VaultService {
        VaultService::new(Arc::new(InMemoryVaultBackend::new()))
    }

    #[test]
    fn initialize_and_unlock_roundtrip() {
        let svc = make_service();
        block_on(async {
            assert!(!svc.is_initialized().await.unwrap());
            svc.initialize("master-pw").await.unwrap();
            assert!(svc.is_initialized().await.unwrap());
            assert!(svc.is_unlocked());

            svc.lock();
            assert!(!svc.is_unlocked());

            svc.unlock("master-pw").await.unwrap();
            assert!(svc.is_unlocked());
        });
    }

    #[test]
    fn wrong_password_fails_unlock() {
        let svc = make_service();
        block_on(async {
            svc.initialize("correct").await.unwrap();
            svc.lock();
            let result = svc.unlock("wrong").await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn store_and_retrieve_credential() {
        let svc = make_service();
        block_on(async {
            svc.initialize("pw").await.unwrap();
            svc.store("steam", "api_key", b"my-secret-key")
                .await
                .unwrap();
            let retrieved = svc.retrieve("steam", "api_key").await.unwrap();
            assert_eq!(retrieved, b"my-secret-key");
        });
    }

    #[test]
    fn store_retrieve_delete_lifecycle() {
        let svc = make_service();
        block_on(async {
            svc.initialize("pw").await.unwrap();
            svc.store("dlsite", "cookie", b"session-data")
                .await
                .unwrap();

            let platforms = svc.list_platforms().await.unwrap();
            assert_eq!(platforms, vec!["dlsite".to_string()]);

            svc.delete("dlsite", "cookie").await.unwrap();
            let result = svc.retrieve("dlsite", "cookie").await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn operations_fail_when_locked() {
        let svc = make_service();
        block_on(async {
            svc.initialize("pw").await.unwrap();
            svc.lock();
            let store_result = svc.store("steam", "key", b"data").await;
            assert!(store_result.is_err());
            let retrieve_result = svc.retrieve("steam", "key").await;
            assert!(retrieve_result.is_err());
        });
    }

    #[test]
    fn double_initialize_fails() {
        let svc = make_service();
        block_on(async {
            svc.initialize("pw").await.unwrap();
            let result = svc.initialize("pw2").await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn unlock_before_init_fails() {
        let svc = make_service();
        block_on(async {
            let result = svc.unlock("pw").await;
            assert!(result.is_err());
        });
    }
}
