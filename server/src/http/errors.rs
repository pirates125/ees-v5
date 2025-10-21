use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Yetkilendirme hatası: {0}")]
    Unauthorized(String),
    
    #[error("Sunucu hatası: {0}")]
    InternalServerError(String),
    
    #[error("Login başarısız: {0}")]
    LoginFailed(String),
    
    #[error("Form validasyon hatası: {0}")]
    FormValidation(String),
    
    #[error("Erişim engellendi: {0}")]
    Blocked(String),
    
    #[error("İnsan müdahalesi gerekiyor: {0}")]
    HumanActionRequired(String),
    
    #[error("Zaman aşımı: {0}")]
    Timeout(String),
    
    #[error("Parse hatası: {0}")]
    ParseError(String),
    
    #[error("Provider aktif değil: {0}")]
    ProviderInactive(String),
    
    #[error("WebDriver hatası: {0}")]
    WebDriverError(String),
    
    #[error("Bilinmeyen hata: {0}")]
    Unknown(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    Unauthorized,
    InternalServerError,
    LoginFailed,
    FormValidation,
    Blocked,
    HumanActionRequired,
    Timeout,
    ParseError,
    ProviderInactive,
    WebDriverError,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub request_id: Option<String>,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    fn to_error_code(&self) -> ErrorCode {
        match self {
            ApiError::Unauthorized(_) => ErrorCode::Unauthorized,
            ApiError::InternalServerError(_) => ErrorCode::InternalServerError,
            ApiError::LoginFailed(_) => ErrorCode::LoginFailed,
            ApiError::FormValidation(_) => ErrorCode::FormValidation,
            ApiError::Blocked(_) => ErrorCode::Blocked,
            ApiError::HumanActionRequired(_) => ErrorCode::HumanActionRequired,
            ApiError::Timeout(_) => ErrorCode::Timeout,
            ApiError::ParseError(_) => ErrorCode::ParseError,
            ApiError::ProviderInactive(_) => ErrorCode::ProviderInactive,
            ApiError::WebDriverError(_) => ErrorCode::WebDriverError,
            ApiError::Unknown(_) => ErrorCode::Unknown,
        }
    }
    
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::LoginFailed(_) => StatusCode::UNAUTHORIZED,
            ApiError::FormValidation(_) => StatusCode::BAD_REQUEST,
            ApiError::Blocked(_) => StatusCode::FORBIDDEN,
            ApiError::HumanActionRequired(_) => StatusCode::PRECONDITION_FAILED,
            ApiError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            ApiError::ParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ProviderInactive(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::WebDriverError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.to_error_code();
        let message = self.to_string();
        
        let error_response = ErrorResponse {
            request_id: None,
            error: ErrorDetail {
                code,
                message,
                details: None,
            },
        };
        
        (status, Json(error_response)).into_response()
    }
}

impl From<fantoccini::error::CmdError> for ApiError {
    fn from(err: fantoccini::error::CmdError) -> Self {
        ApiError::WebDriverError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::ParseError(err.to_string())
    }
}

