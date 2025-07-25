use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::http::StatusCode;
use actix_web::{
    Error,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};

use crate::guard::{ResponseGuard, Status};
use crate::service::{SanitizeInner, SanitizeService};

/// Http Response Sanitizer
///
/// Useful for sanitizing unwanted error responses produced by
/// other actix-web services.
///
/// # Example
///
/// ```
/// use actix_web::{App, error, Responder, http::StatusCode, web};
/// use actix_sanitize::Sanitizer;
///
/// async fn faulty_service() -> impl Responder {
///     error::InternalError::new(
///         "sensitive error message",
///         StatusCode::INTERNAL_SERVER_ERROR,
///     )
/// }
///
/// let app = App::new()
///     .wrap(Sanitizer::default())
///     .service(
///         web::resource("/broken")
///             .route(web::get().to(faulty_service))
///     );
/// ```
pub struct Sanitizer {
    guards: Vec<Rc<dyn ResponseGuard>>,
}

impl Sanitizer {
    /// Creates a new `Sanitizer` middleware instance.
    ///
    /// Status Codes: [404, 405, 500..600] are configured
    /// to be sanitized by default.
    #[inline]
    pub fn new() -> Self {
        Self::empty()
            .guard(Status::from(StatusCode::NOT_FOUND))
            .guard(Status::from(StatusCode::METHOD_NOT_ALLOWED))
            .guard(Status::from(500..600))
    }

    /// Creates a new `Sanitizer` instance with no pre-existing [`ResponseGuard`] rules.
    #[inline]
    pub fn empty() -> Self {
        Self { guards: Vec::new() }
    }

    /// Assign a [`ResponseGuard`] to the existing `Sanitizer` instance.
    pub fn guard<G: ResponseGuard + 'static>(mut self, guard: G) -> Self {
        self.guards.push(Rc::new(guard));
        self
    }
}

impl Default for Sanitizer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Transform<S, ServiceRequest> for Sanitizer
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = SanitizeService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SanitizeService(Rc::new(SanitizeInner {
            service: Rc::new(service),
            guards: self.guards.clone(),
        }))))
    }
}
