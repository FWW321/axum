use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_valid::ValidRejection;
use thiserror::Error;

use crate::router::ApiResponse;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("Not found")]
    NotFound,
    #[error("Query error: {0}")]
    Query(#[from] QueryRejection),
    #[error("Path error: {0}")]
    Path(#[from] PathRejection),
    #[error("Body error: {0}")]
    JSON(#[from] JsonRejection),
    #[error("Invalid request parameters: {0}")]
    Validation(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("Method not allowed")]
    MethodNotAllowed,
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    // Business error
    #[error("{0}")]
    Biz(String),
    #[error("Internal server error: {0}")]
    InternalServerError(#[from] anyhow::Error),
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Biz(_) => StatusCode::OK,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Query(_) |
            ApiError::Validation(_) |
            ApiError::Path(_) |
            ApiError::JSON(_) |
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) |
            ApiError::JWT(_) => StatusCode::UNAUTHORIZED,
            ApiError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            ApiError::InternalServerError(_) |
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let body = ApiResponse::<()>::err(self.to_string());
        (status_code, axum::Json(body)).into_response()
    }
}

// ValidRejection<E>是ValidationRejection<V, E>的别名
// V是验证失败时的值类型
// E是内部抽取器的错误类型
impl From<ValidRejection<ApiError>> for ApiError {
    fn from(value: ValidRejection<ApiError>) -> Self {
        match value {
            ValidRejection::Valid(v) => ApiError::Validation(format!("Validation error: {}", v)),
            ValidRejection::Inner(e) => e,
        }
    }
}

impl From<ApiError> for Response {
    fn from(err: ApiError) -> Self {
        err.into_response()
    }
}
