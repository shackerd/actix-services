use std::{ops::Deref, rc::Rc};

use actix_web::{
    body::BoxBody,
    dev::{self, Service, ServiceRequest, ServiceResponse},
    error::Error as ActixError,
};
use futures_core::future::LocalBoxFuture;

use crate::Authenticator;

/// Assembled Authn service
#[derive(Clone)]
pub struct AuthnService<A, S>(pub(crate) Rc<AuthnInner<A, S>>);

impl<A, S> Deref for AuthnService<A, S> {
    type Target = AuthnInner<A, S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct AuthnInner<A, S> {
    pub(crate) authn: Rc<A>,
    pub(crate) service: Rc<S>,
}

impl<A, S> Service<ServiceRequest> for AuthnService<A, S>
where
    A: Authenticator + 'static,
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = ActixError> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::always_ready!();

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let this = Rc::clone(&self.0);
        Box::pin(async move {
            if !this.authn.authorize(req.request()).await? {
                let res = this.authn.prompt(req.request()).await?;
                return Ok(req.into_response(res));
            }
            this.service.call(req).await
        })
    }
}
