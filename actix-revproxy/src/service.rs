use std::{ops::Deref, rc::Rc};

use actix_web::{
    HttpRequest,
    body::BoxBody,
    dev::{self, Service, ServiceRequest, ServiceResponse},
    error::Error as ActixError,
};
use awc::{
    Client, ClientRequest,
    http::{Uri, header},
};
use futures_core::future::LocalBoxFuture;

use crate::error::Error;
use crate::proxy::*;

pub type HeaderVec = Vec<(header::HeaderName, header::HeaderValue)>;

/// Assembled reverse-proxy service
#[derive(Clone)]
pub struct ProxyService(pub(crate) Rc<ProxyServiceInner>);

impl ProxyService {
    /// Convert [`actix_web::HttpRequest`] into [`awc::ClientRequest`]
    #[inline]
    fn prepare_request(&self, req: &HttpRequest) -> Result<ClientRequest, Error> {
        let info = req.connection_info().clone();
        let uri = combine_uri(&self.resolve, req.uri())?;

        let mut request = req.client_req(&self.client, uri)?.no_decompress();
        if !self.change_host {
            request = request.insert_header((header::HOST, info.host()))
        }

        if let Some(addr) = req.peer_addr() {
            let ip = addr.ip().to_string();
            let proto = request.get_uri().scheme_str().unwrap_or("http").to_owned();
            request = request
                .insert_header((header::X_FORWARDED_HOST, info.host()))
                .insert_header((header::X_FORWARDED_PROTO, proto));
            update_forwarded(request.headers_mut(), header::X_FORWARDED_FOR, ip)?;
        }

        for (name, value) in self.header_up.clone() {
            match value.is_empty() {
                true => request.headers_mut().remove(name),
                false => request.headers_mut().insert(name, value),
            };
        }
        Ok(request)
    }
}

impl Deref for ProxyService {
    type Target = ProxyServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ProxyServiceInner {
    pub(crate) client: Rc<Client>,
    pub(crate) resolve: Uri,
    pub(crate) change_host: bool,
    pub(crate) header_up: HeaderVec,
    pub(crate) header_down: HeaderVec,
}

impl Service<ServiceRequest> for ProxyService {
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::always_ready!();

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            let (http_req, payload) = req.into_parts();

            let addr = http_req
                .peer_addr()
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "<unknown>".to_owned());
            let request = this.prepare_request(&http_req)?;

            tracing::debug!("{addr} {:?} {:?}", http_req.method(), request.get_uri());
            tracing::trace!(?addr, ?request);
            let response = request
                .send_stream(payload)
                .await
                .map_err(Error::FailedRequest)?;
            tracing::trace!(?addr, ?response);

            let mut http_res = response.server_response()?;
            for (name, value) in this.header_down.clone() {
                match value.is_empty() {
                    true => http_res.headers_mut().remove(name),
                    false => http_res.headers_mut().insert(name, value),
                };
            }
            Ok(ServiceResponse::new(http_req, http_res))
        })
    }
}
