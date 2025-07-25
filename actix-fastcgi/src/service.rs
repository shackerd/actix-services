use std::{
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

use actix_files::PathBufWrap;
use actix_web::{
    HttpRequest,
    body::BoxBody,
    dev::{self, Service, ServiceRequest, ServiceResponse},
    error::Error,
};
use fastcgi_client::{Client, Params, Request};
use futures_core::future::LocalBoxFuture;

use super::payload::{RequestStream, ResponseStream};
use super::stream::SockStream;

/// Assembled fastcgi client service
#[derive(Clone)]
pub struct FastCGIService(pub(crate) Rc<FastCGIInner>);

impl FastCGIService {
    /// Fill Additional Paramters from Service Settings and Request Headers
    ///
    /// # Argument Order
    /// The first argument (`root`) is the canonical rooted path for the script
    ///
    /// The second argument (`path`) is the valid uri path generated from the uri
    ///
    /// The third argument (`req`) is the http-request object to load data from
    pub fn fill_params<'a>(&'a self, root: &'a Path, path: &Path, req: &HttpRequest) -> Params<'a> {
        let saddr = req.app_config().local_addr();
        let script_name = format!("/{}", path.to_string_lossy());
        let mut params = Params::default()
            .document_uri(script_name.clone())
            .document_root(self.root.to_string_lossy())
            .request_method(req.method().as_str().to_owned())
            .request_uri(req.uri().path().to_owned())
            .script_name(script_name)
            .script_filename(root.to_string_lossy())
            .server_name(req.connection_info().host().to_owned())
            .server_addr(saddr.ip().to_string())
            .server_port(saddr.port());

        for (name, value) in req.headers() {
            let val = match value.to_str() {
                Ok(val) => val,
                Err(_) => continue,
            };
            let name = match name.as_str() {
                "content-type" => "CONTENT_TYPE".to_owned(),
                "content-length" => "CONTENT_LENGTH".to_owned(),
                name => format!("HTTP_{}", name.replace("-", "_").to_uppercase()),
            };
            params.insert(name.into(), val.to_owned().into());
        }

        if let Some(peer) = req.peer_addr() {
            let client = peer.ip().to_string();
            params = params.remote_addr(client).remote_port(peer.port());
        }
        params
    }
}

impl Deref for FastCGIService {
    type Target = FastCGIInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FastCGIInner {
    pub(crate) root: PathBuf,
    pub(crate) fastcgi_address: String,
}

impl Service<ServiceRequest> for FastCGIService {
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::always_ready!();

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            let path_on_disk = PathBufWrap::parse_req(req.request(), false)?;

            let root = this.root.join(&path_on_disk);
            let params = this.fill_params(&root, path_on_disk.as_ref(), req.request());

            let sock = SockStream::connect(&this.fastcgi_address).await?;
            let client = Client::new(sock);

            let stream = RequestStream::from_request(&mut req);
            let request = Request::new(params, stream.into_reader());

            let stream = client.execute_once_stream(request).await.unwrap();
            let http_res = ResponseStream::new(stream).into_response().await?;

            Ok(req.into_response(http_res))
        })
    }
}
