use std::{ops::Deref, rc::Rc};

use actix_web::{
    body::{BoxBody, to_bytes_limited},
    dev::{self, Service, ServiceRequest, ServiceResponse},
    error::Error as ActixError,
};
use futures_core::future::LocalBoxFuture;

use crate::{error::SanitizedError, guard::ResponseGuard};

/// Assembled response sanitization service
#[derive(Clone)]
pub struct SanitizeService<S>(pub(crate) Rc<SanitizeInner<S>>);

impl<S> Deref for SanitizeService<S> {
    type Target = SanitizeInner<S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct SanitizeInner<S> {
    pub(crate) service: Rc<S>,
    pub(crate) guards: Vec<Rc<dyn ResponseGuard>>,
}

impl<S> Service<ServiceRequest> for SanitizeService<S>
where
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
            let res = this.service.call(req).await.map_err(|err| {
                tracing::error!("captured: {err:?}");
                SanitizedError::from(err)
            })?;

            let (http_req, mut http_res) = res.into_parts();
            if let Some(err) = http_res.error() {
                tracing::error!("sanitized: {err}");
                http_res = SanitizedError::empty(http_res);
                return Ok(ServiceResponse::new(http_req, http_res));
            }

            if this.guards.iter().any(|guard| guard.guard(&http_res)) {
                tracing::trace!(?http_res, "sanitized output");

                let (res, content) = http_res.into_parts();
                let message = to_bytes_limited(content, 250).await;
                tracing::error!("sanitized: {message:?}");

                http_res = SanitizedError::empty(res);
            }
            Ok(ServiceResponse::new(http_req, http_res))
        })
    }
}
