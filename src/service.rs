use std::{ops::Deref, path::PathBuf, rc::Rc};

use actix_files::PathBufWrap;
use actix_web::{
    body::BoxBody,
    dev::{self, Service, ServiceRequest, ServiceResponse},
    error::Error,
};
use fastcgi_client::{Client, Params, Request};
use futures_core::future::LocalBoxFuture;

use super::payload::StreamBuf;
use super::stream::SockStream;

/// Server Address Type Alias
pub type Addr = (String, u16);

#[derive(Clone)]
pub struct FastCGIService(pub(crate) Rc<FastCGIInner>);

impl Deref for FastCGIService {
    type Target = FastCGIInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FastCGIInner {
    pub(crate) root: PathBuf,
    pub(crate) fastcgi_address: String,
    pub(crate) server_address: Option<Addr>,
}

impl Service<ServiceRequest> for FastCGIService {
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::always_ready!();

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            let path_on_disk = PathBufWrap::parse_req(req.request(), false)?;

            let path = this.root.join(&path_on_disk);
            let script_name = path_on_disk.as_ref().to_string_lossy();
            let mut params = Params::default()
                .document_uri(script_name.clone())
                .document_root(this.root.to_string_lossy())
                .request_method(req.method().as_str())
                .request_uri(req.uri().path())
                .script_name(script_name)
                .script_filename(path.to_string_lossy())
                .server_name(req.connection_info().host().to_owned());

            if let Some((host, port)) = this.server_address.as_ref() {
                params = params.server_addr(host).server_port(*port)
            }
            if let Some(peer) = req.peer_addr() {
                let client = peer.ip().to_string();
                params = params.remote_addr(client).remote_port(peer.port());
            }

            let sock = SockStream::connect(&this.fastcgi_address).await?;
            let client = Client::new(sock);

            let empty = tokio::io::empty();
            let request = Request::new(params, empty);

            let stream = client.execute_once_stream(request).await.unwrap();
            let http_res = StreamBuf::new(stream).into_response().await?;

            Ok(req.into_response(http_res))
        })
    }
}
