use std::ops::Deref;
use std::rc::Rc;

use actix_web::{
    Error as ActixError,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, forward_ready},
};
use futures_core::future::LocalBoxFuture;

use crate::modsecurity::ModSecurity;

/// Assembled LibModSecurity service
#[derive(Clone)]
pub struct ModSecurityService<S>(pub(crate) Rc<ModSecurityInner<S>>);

impl<S> Deref for ModSecurityService<S> {
    type Target = ModSecurityInner<S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ModSecurityInner<S> {
    pub(crate) service: Rc<S>,
    pub(crate) modsecurity: Rc<ModSecurity>,
}

impl<S> Service<ServiceRequest> for ModSecurityService<S>
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
            let mut transaction = this.modsecurity.transaction()?;
            transaction.process_request(&mut req).await?;

            if let Some(intv) = transaction.intervention()? {
                return Ok(req.into_response(intv));
            }

            let res = this.service.call(req).await?;

            let (http_req, mut http_res) = res.into_parts();
            http_res = transaction.process_response(http_res).await?;

            match transaction.intervention()? {
                Some(intv) => Ok(ServiceResponse::new(http_req, intv.into())),
                None => Ok(ServiceResponse::new(http_req, http_res)),
            }
        })
    }
}
