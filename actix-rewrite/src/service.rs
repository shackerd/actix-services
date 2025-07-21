use std::{ops::Deref, rc::Rc, str::FromStr};

use actix_http::Uri;
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, forward_ready},
    error::Error as ActixError,
};
use futures_core::future::LocalBoxFuture;

use super::error::Error;
use super::rewrite::{Engine, Rewrite};
use super::util;

/// Assembled `mod_rewrite` service
#[derive(Clone)]
pub struct RewriteService<S>(pub(crate) Rc<RewriteInner<S>>);

impl<S> Deref for RewriteService<S> {
    type Target = RewriteInner<S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RewriteInner<S> {
    pub(crate) service: Rc<S>,
    pub(crate) engine: Rc<Engine>,
}

impl<S> Service<ServiceRequest> for RewriteService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = ActixError> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let this = Rc::clone(&self.0);
        Box::pin(async move {
            let uri = match this.engine.rewrite(req.request())? {
                Rewrite::Uri(uri) => uri,
                Rewrite::Redirect(res) => return Ok(req.into_response(res)),
                Rewrite::Response(res) => return Ok(req.into_response(res)),
            };

            let after = Uri::from_str(&uri).map_err(Error::InvalidUri)?;
            req.head_mut().uri = util::join_uri(req.uri(), &after)?;

            this.service.call(req).await
        })
    }
}
