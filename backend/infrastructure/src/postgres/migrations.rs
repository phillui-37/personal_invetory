#[cfg(feature = "postgres")]
use sqlx::PgPool;

use domain::DomainError;

#[cfg(feature = "postgres")]
pub async fn run_migrations(pool: &PgPool) -> Result<(), DomainError> {
    sqlx::migrate!("./migrations_pg")
        .run(pool)
        .await
        .map_err(|e| DomainError::InternalError(format!("PG migration failed: {e}")))
}

#[cfg(not(feature = "postgres"))]
pub async fn run_migrations(_pool: &()) -> Result<(), DomainError> {
    Err(DomainError::ValidationError(
        "postgres feature is not enabled".to_string(),
    ))
}
