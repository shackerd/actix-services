use std::fmt::Display;

use actix_web::{Error, HttpResponse, ResponseError, body::BoxBody};

#[derive(Debug)]
pub struct SanitizedError(Error);

impl SanitizedError {
    pub(crate) fn empty<B>(res: HttpResponse<B>) -> HttpResponse {
        res.set_body(BoxBody::new(""))
    }
}

impl Display for SanitizedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Error> for SanitizedError {
    #[inline]
    fn from(value: Error) -> Self {
        Self(value)
    }
}

impl ResponseError for SanitizedError {
    #[inline]
    fn error_response(&self) -> HttpResponse<BoxBody> {
        Self::empty(self.0.error_response())
    }
}
