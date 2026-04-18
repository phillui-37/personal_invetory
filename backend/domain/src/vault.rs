use async_trait::async_trait;
use crate::DomainError;

#[derive(Debug, Clone)]
pub struct VaultConfig {
    pub salt: Vec<u8>,
    pub key_check: Vec<u8>,
    pub key_check_nonce: Vec<u8>,
}

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
