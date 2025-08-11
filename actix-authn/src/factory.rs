use std::{
    future::{Ready, ready},
    rc::Rc,
};

use actix_web::{
    Error,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};

use crate::{Authenticator, AuthnService, service::AuthnInner};

/// Authn middleware service
///
/// `Authn` must be registered with `App::wrap()` method.
///
/// `Authn` supports any object that implements the [`Authenticator`] trait.
///
/// # Example
///
/// ```
/// use actix_web::App;
/// use actix_authn::{Authn, basic::{Basic, Passwd}};
///
/// /// passwords should be generated outside of HttpServer::new
/// /// or use [`Basic::passwd`] or [`Basic::htpasswd`].
/// let passwd = Passwd::new("admin");
///
/// let app = App::new()
///     .wrap(Authn::new(Basic::default().auth("admin", passwd).build()));
/// ```
pub struct Authn<A>(Rc<A>);

impl<A> Authn<A>
where
    A: Authenticator,
{
    /// Creates a new `ModSecurity` middleware instance
    #[inline]
    pub fn new(authn: A) -> Self {
        Self(Rc::new(authn))
    }
}

impl<A, S> Transform<S, ServiceRequest> for Authn<A>
where
    A: Authenticator + 'static,
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthnService<A, S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthnService(Rc::new(AuthnInner {
            authn: Rc::clone(&self.0),
            service: Rc::new(service),
        }))))
    }
}
