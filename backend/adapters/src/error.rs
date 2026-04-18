use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::DomainError;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: message.into(),
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::NotFound(message) => Self {
                status: StatusCode::NOT_FOUND,
                message,
            },
            DomainError::ValidationError(message) => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                message,
            },
            DomainError::Conflict(message) => Self {
                status: StatusCode::CONFLICT,
                message,
            },
            DomainError::InternalError(message) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}
