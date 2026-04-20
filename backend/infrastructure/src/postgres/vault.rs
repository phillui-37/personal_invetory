use async_trait::async_trait;
use domain::vault::{VaultBackend, VaultConfig};
use domain::DomainError;
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {crate::postgres::error::pg_err, sqlx::{PgPool, Row}};

#[cfg(feature = "postgres")]
pub struct PgVaultBackend {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgVaultBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl VaultBackend for PgVaultBackend {
    async fn get_config(&self) -> Result<Option<VaultConfig>, DomainError> {
        let row = sqlx::query(
            "SELECT salt, key_check, key_check_nonce FROM vault_config WHERE id = 'default'",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(row.map(|r| VaultConfig {
            salt: r.try_get("salt").unwrap_or_default(),
            key_check: r.try_get("key_check").unwrap_or_default(),
            key_check_nonce: r.try_get("key_check_nonce").unwrap_or_default(),
        }))
    }

    async fn save_config(&self, config: &VaultConfig) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO vault_config (id, salt, key_check, key_check_nonce)
             VALUES ('default', $1, $2, $3)
             ON CONFLICT (id) DO UPDATE SET salt = $1, key_check = $2, key_check_nonce = $3",
        )
        .bind(&config.salt)
        .bind(&config.key_check)
        .bind(&config.key_check_nonce)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(())
    }

    async fn store_blob(
        &self,
        platform: &str,
        credential_type: &str,
        encrypted_blob: &[u8],
        nonce: &[u8],
    ) -> Result<(), DomainError> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO credentials (id, platform, credential_type, encrypted_blob, nonce)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (platform, credential_type)
             DO UPDATE SET encrypted_blob = $4, nonce = $5, updated_at = now()",
        )
        .bind(&id)
        .bind(platform)
        .bind(credential_type)
        .bind(encrypted_blob)
        .bind(nonce)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(())
    }

    async fn retrieve_blob(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(Vec<u8>, Vec<u8>), DomainError> {
        let row = sqlx::query(
            "SELECT encrypted_blob, nonce FROM credentials
             WHERE platform = $1 AND credential_type = $2",
        )
        .bind(platform)
        .bind(credential_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| {
            DomainError::NotFound(format!("no credential for {platform}/{credential_type}"))
        })?;
        let blob: Vec<u8> = row.try_get("encrypted_blob").map_err(pg_err)?;
        let nonce: Vec<u8> = row.try_get("nonce").map_err(pg_err)?;
        Ok((blob, nonce))
    }

    async fn delete_credential(
        &self,
        platform: &str,
        credential_type: &str,
    ) -> Result<(), DomainError> {
        sqlx::query(
            "DELETE FROM credentials WHERE platform = $1 AND credential_type = $2",
        )
        .bind(platform)
        .bind(credential_type)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;
        Ok(())
    }

    async fn list_platforms(&self) -> Result<Vec<String>, DomainError> {
        let rows = sqlx::query("SELECT DISTINCT platform FROM credentials ORDER BY platform")
            .fetch_all(&self.pool)
            .await
            .map_err(pg_err)?;
        rows.iter()
            .map(|r| r.try_get::<String, _>("platform").map_err(pg_err))
            .collect()
    }
}

#[cfg(not(feature = "postgres"))]
pub fn _placeholder() {}

