use crate::sqlite::{map_sqlite_error, SharedSqliteConnection};
use async_trait::async_trait;
use domain::{DomainError, GameMeta, GameMetaRepository, NewGameMeta};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

pub struct SqliteGameMetaRepository {
    conn: SharedSqliteConnection,
}

impl SqliteGameMetaRepository {
    pub fn new(conn: SharedSqliteConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl GameMetaRepository for SqliteGameMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<GameMeta, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.query_row(
            "SELECT platform, store, developer, publisher, manual_notes
             FROM game_metas
             WHERE resource_id = ?1",
            params![resource_id.to_string()],
            |row| {
                Ok(GameMeta {
                    resource_id,
                    platform: row.get(0)?,
                    store: row.get(1)?,
                    developer: row.get(2)?,
                    publisher: row.get(3)?,
                    manual_notes: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(map_sqlite_error)?
        .ok_or_else(|| DomainError::NotFound(format!("game meta {resource_id} not found")))
    }

    async fn upsert(&self, resource_id: Uuid, input: NewGameMeta) -> Result<GameMeta, DomainError> {
        let platform = input.platform;
        let store = input.store;
        let developer = input.developer;
        let publisher = input.publisher;
        let manual_notes = input.manual_notes;
        let conn = self
            .conn
            .lock()
            .map_err(|_| DomainError::InternalError("sqlite connection mutex poisoned".to_string()))?;

        conn.execute(
            "INSERT INTO game_metas (resource_id, platform, store, developer, publisher, manual_notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(resource_id) DO UPDATE SET
                 platform = excluded.platform,
                 store = excluded.store,
                 developer = excluded.developer,
                 publisher = excluded.publisher,
                 manual_notes = excluded.manual_notes",
            params![
                resource_id.to_string(),
                platform.as_deref(),
                store.as_deref(),
                developer.as_deref(),
                publisher.as_deref(),
                manual_notes.as_deref()
            ],
        )
        .map_err(map_sqlite_error)?;
        Ok(GameMeta {
            resource_id,
            platform,
            store,
            developer,
            publisher,
            manual_notes,
        })
    }
}
