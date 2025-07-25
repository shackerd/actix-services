//! [`ResponseGuard`] Implementation and Utilities
use std::ops::Range;

use actix_web::{HttpResponse, http::StatusCode};

/// Response equivalent of [`actix_web::guard::Guard`].
///
/// Defines logic for matching content that should be sanitized.
pub trait ResponseGuard {
    fn guard(&self, res: &HttpResponse) -> bool;
}

/// Simple status-code matcher using a range of numbers.
pub struct Status(Range<u16>);

impl From<u16> for Status {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value..value)
    }
}

impl From<Range<u16>> for Status {
    #[inline]
    fn from(value: Range<u16>) -> Self {
        Self(value)
    }
}

impl From<StatusCode> for Status {
    #[inline]
    fn from(value: StatusCode) -> Self {
        Self::from(value.as_u16())
    }
}

impl ResponseGuard for Status {
    #[inline]
    fn guard(&self, res: &HttpResponse) -> bool {
        self.0.contains(&res.status().as_u16())
    }
}
