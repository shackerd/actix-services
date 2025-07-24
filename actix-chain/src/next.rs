//! All tools and utilities related to [`Link::next`](crate::Link::next)

use std::rc::Rc;

use actix_web::{
    HttpResponse,
    http::{StatusCode, header::HeaderName},
};

/// Response equivalent of [`actix_web::guard::Guard`].
///
/// Blocks responses that contain matching criteria
/// and allows the request to be forwarded to the next
/// [`Link`](crate::Link) in the [`Chain`](crate::Chain).
pub trait Next {
    fn next(&self, res: &HttpResponse) -> bool;
}

/// Simple [`StatusCode`] response guard.
///
/// Blocks the response the specified status-code is present.
pub struct IsStatus(pub StatusCode);

impl IsStatus {
    #[inline]
    pub fn new(status: StatusCode) -> Self {
        Self(status)
    }
    #[inline]
    pub(crate) fn rc(status: StatusCode) -> Rc<Self> {
        Rc::new(Self::new(status))
    }
}

impl From<StatusCode> for IsStatus {
    #[inline]
    fn from(value: StatusCode) -> Self {
        Self::new(value)
    }
}

impl Next for IsStatus {
    #[inline]
    fn next(&self, res: &HttpResponse) -> bool {
        res.status() == self.0
    }
}

/// Simple [`HeaderName`]
/// response guard.
///
/// Blocks the response if the specified header is present.
pub struct HasHeader(pub HeaderName);

impl HasHeader {
    pub fn new(name: HeaderName) -> Self {
        Self(name)
    }
}

impl From<HeaderName> for HasHeader {
    #[inline]
    fn from(value: HeaderName) -> Self {
        Self::new(value)
    }
}

impl Next for HasHeader {
    #[inline]
    fn next(&self, res: &HttpResponse) -> bool {
        res.headers().contains_key(&self.0)
    }
}
