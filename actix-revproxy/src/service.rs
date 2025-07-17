use std::{ops::Deref, rc::Rc};

use actix_web::{
    HttpMessage, HttpResponseBuilder,
    body::BoxBody,
    dev::{self, Service, ServiceRequest, ServiceResponse},
    error::Error as ActixError,
};
use awc::{
    Client,
    http::{Uri, header::HeaderName},
};
use futures_core::future::LocalBoxFuture;

use crate::error::Error;

use super::proxy::*;

/// Assembled reverse-proxy service
#[derive(Clone)]
pub struct ProxyService(pub(crate) Rc<ProxyServiceInner>);

impl Deref for ProxyService {
    type Target = ProxyServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ProxyServiceInner {
    pub(crate) client: Rc<Client>,
    pub(crate) resolve: Uri,
    pub(crate) forward: Option<HeaderName>,
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
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| "<unknown>".to_owned());
            tracing::debug!("{addr} {:?}", http_req.uri());

            let uri = combine_uri(&this.resolve, http_req.uri())?;
            let mut request = this
                .client
                .request(http_req.method().clone(), uri)
                .no_decompress();

            for header in http_req.headers() {
                request = request.append_header(header);
            }
            remove_connection_headers(request.headers_mut())?;
            remove_hop_headers(request.headers_mut());

            if let Some(forward) = this.forward.as_ref() {
                if !addr.is_empty() {
                    update_forwarded(request.headers_mut(), forward.clone(), addr.clone())?;
                }
            }

            tracing::trace!(?addr, ?request);
            let mut response = request
                .send_stream(payload)
                .await
                .map_err(|err| Error::FailedRequest(err))?;
            tracing::trace!(?addr, ?response);

            let payload = response.take_payload();
            let body = actix_web::body::BodyStream::new(payload);

            let mut builder = HttpResponseBuilder::new(response.status());
            for header in response.headers() {
                builder.append_header(header);
            }

            let mut http_res = builder.body(body);
            remove_connection_headers(http_res.headers_mut())?;
            remove_hop_headers(http_res.headers_mut());

            Ok(ServiceResponse::new(http_req, http_res))
        })
    }
}
