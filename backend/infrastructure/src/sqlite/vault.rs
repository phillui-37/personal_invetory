use async_trait::async_trait;
use uuid::Uuid;

use domain::vault::{VaultBackend, VaultConfig};
use domain::DomainError;

use super::map_sqlite_error;
use super::SharedSqliteConnection;

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
