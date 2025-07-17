//! Error and Result module

use actix_web::{ResponseError, error::QueryPayloadError};
use awc::http::header::{InvalidHeaderValue, ToStrError};
use derive_more::{Display, Error, From};

/// Errors which occur when processing Reverse Proxy Requests/Responses
#[derive(Debug, Display, From, Error)]
#[non_exhaustive]
pub enum Error {
    /// Unexpected IO Error
    Io(std::io::Error),

    /// Proxy client request failed
    FailedRequest(awc::error::SendRequestError),

    /// Failed to parse value during header processing
    InvalidHeader(ToStrError),

    /// Failed to append new header
    InvalidHeaderValue(InvalidHeaderValue),
}

/// Errors which occur when building a combined proxied request uri
#[derive(Debug, Display, From, Error)]
pub enum UriError {
    #[display("Failed to combine URI paths")]
    InvalidUriPath,

    #[display("Missing proxy url authority")]
    MissingAuthority,

    #[display("Invalid query string supplied")]
    InvalidQuery(QueryPayloadError),

    #[display("Failed to encode combined query string")]
    QueryEncoderError(serde_urlencoded::ser::Error),

    #[display("Failed to build http request uri")]
    RequestError(awc::error::HttpError),
}

impl ResponseError for Error {
    /// Returns `500 Internal Server Error`.
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl ResponseError for UriError {
    /// Returns `500 Internal Server Error`.
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
