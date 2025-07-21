use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::{
    Error,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};

use crate::rewrite::Engine;
use crate::service::{RewriteInner, RewriteService};

/// `mod_rewrite` middleware service
///
/// `Middleware` must be registered with `App::wrap()` method.
///
/// # Examples
///
/// ```
/// use actix_web::App;
/// use actix_rewrite::{Middleware, Engine};
///
/// let mut engine = Engine::new();
/// engine.add_rules(r#"
///     RewriteRule /file/(.*)     /tmp/$1      [L]
///     RewriteRule /redirect/(.*) /location/$1 [R=302]
///     RewriteRule /blocked/(.*)  -            [F]
/// "#).expect("failed to process rules");
///
/// let app = App::new().wrap(Middleware::new(engine));
/// ```
pub struct Middleware(Rc<Engine>);

impl Middleware {
    /// Creates a new `mod_rewrite` middleware instance
    #[inline]
    pub fn new(engine: Engine) -> Self {
        Self(Rc::new(engine))
    }
}

impl From<Engine> for Middleware {
    #[inline]
    fn from(value: Engine) -> Self {
        Self::new(value)
    }
}

impl<S> Transform<S, ServiceRequest> for Middleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = RewriteService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RewriteService(Rc::new(RewriteInner {
            service: Rc::new(service),
            engine: self.0.clone(),
        }))))
    }
}
