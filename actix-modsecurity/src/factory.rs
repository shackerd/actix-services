use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::{
    Error,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};

use crate::ModSecurityService;
use crate::builder::Builder;
use crate::modsecurity::ModSecurity;
use crate::service::ModSecurityInner;

/// ModSecurity middleware service
///
/// `Middleware` must be registered with `App::wrap()` method.
///
/// # Examples
///
/// ```
/// use actix_web::App;
/// use actix_modsecurity::{Middleware, ModSecurity};
///
/// let mut security = ModSecurity::new();
/// security.add_rules(r#"
///  SecRuleEngine On
///  SecRule REQUEST_URI "@rx admin" "id:1,phase:1,deny,status:401"
/// "#).expect("Failed to add rules");
///
/// let app = App::new().wrap(Middleware::new(security));
/// ```
pub struct Middleware(Rc<ModSecurity>);

impl Middleware {
    /// Creates a new `ModSecurity` middleware instance
    #[inline]
    pub fn new(modsecurity: ModSecurity) -> Self {
        Self(Rc::new(modsecurity))
    }
}

impl From<ModSecurity> for Middleware {
    #[inline]
    fn from(value: ModSecurity) -> Self {
        Self::new(value)
    }
}

impl From<Builder> for Middleware {
    #[inline]
    fn from(value: Builder) -> Self {
        Self::new(value.build())
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
    type Transform = ModSecurityService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ModSecurityService(Rc::new(ModSecurityInner {
            service: Rc::new(service),
            modsecurity: Rc::clone(&self.0),
        }))))
    }
}
