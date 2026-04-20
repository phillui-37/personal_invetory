#[cfg(feature = "postgres")]
use domain::DomainError;

#[cfg(feature = "postgres")]
pub fn pg_err(e: sqlx::Error) -> DomainError {
    match e {
        sqlx::Error::RowNotFound => DomainError::NotFound("row not found".to_string()),
        _ => DomainError::InternalError(format!("PG error: {e}")),
    }
}
