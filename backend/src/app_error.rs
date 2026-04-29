use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
pub struct AppError {
    code: StatusCode,
    message: String,
}

impl AppError {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> StatusCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = (self.code, self.message);
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        eprintln!("SQLx error: {:?}", err);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "An unexpected database error occurred",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_new() {
        let err = AppError::new(StatusCode::BAD_REQUEST, "Invalid input");
        assert_eq!(err.code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.message(), "Invalid input");
    }

    #[test]
    fn test_app_error_from_sqlx() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err = AppError::from(sqlx_err);
        assert_eq!(app_err.code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(app_err.message(), "An unexpected database error occurred");
    }
}
