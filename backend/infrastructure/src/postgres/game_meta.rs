use async_trait::async_trait;
use domain::{DomainError, GameMeta, GameMetaRepository, NewGameMeta};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {crate::postgres::error::pg_err, sqlx::{PgPool, Row}};

#[cfg(feature = "postgres")]
pub struct PgGameMetaRepository {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PgGameMetaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl GameMetaRepository for PgGameMetaRepository {
    async fn get(&self, resource_id: Uuid) -> Result<GameMeta, DomainError> {
        let row = sqlx::query(
            "SELECT resource_id, platform, store, developer, publisher, manual_notes \
             FROM game_metas WHERE resource_id = $1",
        )
        .bind(resource_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(pg_err)?
        .ok_or_else(|| {
            DomainError::NotFound(format!("game meta for resource {resource_id} not found"))
        })?;

        let rid: String = row.try_get("resource_id").map_err(pg_err)?;
        Ok(GameMeta {
            resource_id: Uuid::parse_str(&rid)
                .map_err(|e| DomainError::InternalError(e.to_string()))?,
            platform: row.try_get("platform").map_err(pg_err)?,
            store: row.try_get("store").map_err(pg_err)?,
            developer: row.try_get("developer").map_err(pg_err)?,
            publisher: row.try_get("publisher").map_err(pg_err)?,
            manual_notes: row.try_get("manual_notes").map_err(pg_err)?,
        })
    }

    async fn upsert(&self, resource_id: Uuid, input: NewGameMeta) -> Result<GameMeta, DomainError> {
        sqlx::query(
            "INSERT INTO game_metas (resource_id, platform, store, developer, publisher, manual_notes) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT(resource_id) DO UPDATE SET \
               platform = EXCLUDED.platform, \
               store = EXCLUDED.store, \
               developer = EXCLUDED.developer, \
               publisher = EXCLUDED.publisher, \
               manual_notes = EXCLUDED.manual_notes",
        )
        .bind(resource_id.to_string())
        .bind(&input.platform)
        .bind(&input.store)
        .bind(&input.developer)
        .bind(&input.publisher)
        .bind(&input.manual_notes)
        .execute(&self.pool)
        .await
        .map_err(pg_err)?;

        Ok(GameMeta {
            resource_id,
            platform: input.platform,
            store: input.store,
            developer: input.developer,
            publisher: input.publisher,
            manual_notes: input.manual_notes,
        })
    }
}

#[cfg(not(feature = "postgres"))]
#[derive(Default)]
pub struct PgGameMetaRepository;

#[cfg(not(feature = "postgres"))]
#[async_trait]
impl GameMetaRepository for PgGameMetaRepository {
    async fn get(&self, _resource_id: Uuid) -> Result<GameMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
    async fn upsert(&self, _resource_id: Uuid, _input: NewGameMeta) -> Result<GameMeta, DomainError> {
        Err(DomainError::InternalError(
            "postgres feature not enabled".to_string(),
        ))
    }
}
