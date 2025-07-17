//! Error and Result module

use actix_http::error::InvalidStatusCode;
use actix_web::{ResponseError, body::BodyLimitExceeded, error::PayloadError};
use derive_more::{Display, Error, From};

/// Errors which occur when processing FastCGI Requests/Responses
#[derive(Debug, Display, From, Error)]
#[non_exhaustive]
pub enum Error {
    /// Error during LibModSecurity operations
    ModSecurity(modsecurity::error::ModSecurityError),

    /// Request/Response body limit exceeded
    BodyLimitExceeded(BodyLimitExceeded),

    #[display("Failed to process request body")]
    RequestBodyError(PayloadError),

    #[display("Failed to process response body")]
    ResponseBodyError(Box<dyn std::error::Error>),

    #[display("Failed to process response status code")]
    ResponseCodeError(InvalidStatusCode),

    #[display("Failed to build intervention response")]
    ResponseBuildError(actix_web::Error),
}

impl ResponseError for Error {
    /// Returns `500 Internal Server Error`.
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
