use std::{ops::Deref, rc::Rc};

use actix_service::boxed::{BoxService, BoxServiceFactory};
use actix_web::{
    HttpMessage,
    body::BoxBody,
    dev::{self, Service, ServiceRequest, ServiceResponse},
    error::Error,
};
use futures_core::future::LocalBoxFuture;

use crate::link::{LinkInner, default_response};
use crate::payload::PayloadRef;

pub type HttpService = BoxService<ServiceRequest, ServiceResponse, Error>;
pub type HttpNewService = BoxServiceFactory<(), ServiceRequest, ServiceResponse, Error, ()>;

/// Assembled chain service.
#[derive(Clone)]
pub struct ChainService(pub(crate) Rc<ChainInner>);

impl Deref for ChainService {
    type Target = ChainInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ChainInner {
    pub(crate) links: Vec<LinkInner>,
    pub(crate) body_buffer_size: usize,
}

impl Service<ServiceRequest> for ChainService {
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::always_ready!();

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        if self.links.is_empty() {
            tracing::warn!("no links present in chain!");
            return Box::pin(async move { Ok(default_response(req)) });
        }

        let this = self.clone();
        if self.links.len() == 1 {
            return Box::pin(async move { this.links[0].call_once(req).await });
        }

        Box::pin(async move {
            let payload = req.take_payload();
            let buf = PayloadRef::new(payload, this.body_buffer_size);
            req.set_payload(buf.into_payload());

            let ctx = req.guard_ctx();
            let active_links: Vec<_> = this
                .links
                .iter()
                .enumerate()
                .filter(|(_, link)| link.matches(req.uri().path(), &ctx))
                .collect();

            let addr = req
                .peer_addr()
                .map(|addr| addr.to_string())
                .unwrap_or_default();
            tracing::debug!(
                "{addr} {}/{} links matched {:?} {:?}",
                active_links.len(),
                this.links.len(),
                req.method(),
                req.uri()
            );

            let mut link_iter = active_links.into_iter().peekable();
            while let Some((n, link)) = link_iter.next() {
                tracing::debug!("{addr} calling link {n}");
                let mut original_uri = None;
                if let Some(uri) = link.new_uri(req.uri()) {
                    original_uri = Some(req.uri().clone());
                    tracing::debug!("{addr} updated uri {:?} -> {uri:?}", req.uri());
                    req.head_mut().uri = uri;
                }

                let res = link.service.call(req).await?;
                let (http_req, http_res) = res.into_parts();
                tracing::debug!("{addr} link {n} response={:?}", http_res.status());
                if link_iter.peek().is_none() || !link.go_next(&http_res) {
                    return Ok(ServiceResponse::new(http_req, http_res));
                }

                buf.get_mut().reset_stream();
                req = ServiceRequest::from_parts(http_req, buf.into_payload());

                if let Some(uri) = original_uri {
                    req.head_mut().uri = uri;
                }
            }

            Ok(default_response(req))
        })
    }
}
