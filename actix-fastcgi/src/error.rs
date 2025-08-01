//! Error and Result module

use actix_web::{ResponseError, error::PayloadError};
use derive_more::{Display, Error, From};

/// Errors which occur when processing FastCGI Requests/Responses
#[derive(Debug, Display, From, Error)]
#[non_exhaustive]
pub enum Error {
    /// Unexpected IO Error
    Io(std::io::Error),

    /// Stream ended before all http-headers could be read
    #[display("Stream Ended Unexpectedly")]
    UnexpectedEnd,

    /// FastCGI Client Error
    #[display("FastCGI Client Error")]
    ClientError(fastcgi_client::ClientError),

    /// Unknown error within inner stream reader
    #[display("Error when processing stream")]
    Payload(PayloadError),

    /// Error when parsing collected response headers
    #[display("Failed to parse response headers")]
    InvalidHeaders(httparse::Error),

    /// FastCGI Status header code is invalid
    #[display("Invalid status code passed")]
    StatusCode(http::status::InvalidStatusCode),
}

impl From<std::convert::Infallible> for Error {
    fn from(_value: std::convert::Infallible) -> Self {
        panic!("Infallible!")
    }
}

impl ResponseError for Error {
    /// Returns `500 Internal Server Error`.
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
