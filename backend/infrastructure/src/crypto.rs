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
