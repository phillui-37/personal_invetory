#[cfg(feature = "postgres")]
use sqlx::PgPool;

use domain::DomainError;

#[cfg(feature = "postgres")]
pub async fn open_pg_pool(database_url: &str) -> Result<PgPool, DomainError> {
    PgPool::connect(database_url)
        .await
        .map_err(|e| DomainError::InternalError(format!("PG pool connect failed: {e}")))
}

#[cfg(not(feature = "postgres"))]
pub async fn open_pg_pool(_database_url: &str) -> Result<(), DomainError> {
    Err(DomainError::ValidationError(
        "postgres feature is not enabled".to_string(),
    ))
}
