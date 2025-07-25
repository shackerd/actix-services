//! Error and Result module

use actix_web::ResponseError;
use derive_more::{Display, Error, From};

/// Errors which occur when processing Reverse Proxy Requests/Responses
#[derive(Debug, Display, From, Error)]
#[non_exhaustive]
pub enum Error {
    #[display("Internal Io Error")]
    IoError(std::io::Error),

    #[display("Invalid rewrite expression")]
    RuleError(mod_rewrite::error::ExpressionError),

    #[display("Rewrite engine encountered an error")]
    RewriteError(mod_rewrite::error::EngineError),

    #[display("Rewrite returned invalid status code")]
    InvalidStatus(actix_http::error::InvalidStatusCode),

    #[display("Rewrite generated an invalid uri")]
    InvalidUri(actix_http::uri::InvalidUri),

    #[display("Rewrite query join failed to parse query")]
    InvalidQuery(actix_web::error::QueryPayloadError),

    #[display("Failed to decode rewritten url")]
    UrlDecodeError(#[error(not(source))] String),

    #[display("Failed to re-encode query-string")]
    QueryEncodeError(serde_urlencoded::ser::Error),

    #[display("Failed to build http request uri")]
    RequestError(actix_web::error::HttpError),
}

impl ResponseError for Error {
    /// Returns `500 Internal Server Error`.
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
