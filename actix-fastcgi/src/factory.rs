//! FastCGI Service Factory

use std::{path::PathBuf, rc::Rc};

use actix_service::ServiceFactory;
use actix_web::{
    Error,
    dev::{AppService, HttpServiceFactory, ResourceDef, ServiceRequest, ServiceResponse},
    guard::Guard,
};
use futures_core::future::LocalBoxFuture;

use super::service::{FastCGIInner, FastCGIService};

/// FastCGI client service
///
/// `FastCGI` service must be registered with `App::service()` method.
///
/// # Examples
///
/// ```
/// use actix_web::App;
/// use actix_fastcgi::FastCGI;
///
/// let app = App::new()
///     .service(FastCGI::new("/", ".", "tcp://127.0.0.1:9000"));
/// ```
#[derive(Clone)]
pub struct FastCGI {
    mount_path: String,
    guards: Vec<Rc<dyn Guard>>,
    root: PathBuf,
    fastcgi_address: String,
}

impl FastCGI {
    /// Creates new `FastGCI` instance for a specified root directory
    ///
    /// # Argument Order
    /// The first argument (`mount_path`) is the root URL at which the static files are served.
    /// For example, `/assets` will serve files at `example.com/assets/...`.
    ///
    /// The second argument (`root`) is the location on disk at which fastcgi script
    /// files are referenced. For Example, `/index.php` will serve at `example.com/index.php`
    ///
    /// The third argument (`fastcgi_address`) is the tcp/unix socket address for the
    /// fastcgi service.
    ///
    pub fn new<P: Into<PathBuf>>(mount_path: &str, root: P, fastcgi_address: &str) -> Self {
        let root_new = root.into();
        let root = match root_new.canonicalize() {
            Ok(root) => root,
            Err(_) => {
                tracing::error!("Specified root is not a directory: {root_new:?}");
                PathBuf::new()
            }
        };
        Self {
            mount_path: mount_path.to_owned(),
            guards: Vec::new(),
            root,
            fastcgi_address: fastcgi_address.to_owned(),
        }
    }

    /// Adds a routing guard.
    ///
    /// Use this to allow multiple chained services that respond to strictly different
    /// properties of a request. Due to the way routing works, if a guard check returns true and the
    /// request starts being handled by the file service, it will not be able to back-out and try
    /// the next service, you will simply get a 404 (or 405) error response.
    ///
    /// # Examples
    /// ```
    /// use actix_web::{guard::Header, App};
    /// use actix_fastcgi::FastCGI;
    ///
    /// App::new().service(
    ///     FastCGI::new("/","/my/php/files", "unix:///var/run/fastcgi.sock")
    ///         .guard(Header("Host", "example.com"))
    /// );
    /// ```
    pub fn guard<G: Guard + 'static>(mut self, guards: G) -> Self {
        self.guards.push(Rc::new(guards));
        self
    }
}

impl HttpServiceFactory for FastCGI {
    fn register(mut self, config: &mut AppService) {
        let guards = if self.guards.is_empty() {
            None
        } else {
            let guards = std::mem::take(&mut self.guards);
            Some(
                guards
                    .into_iter()
                    .map(|guard| -> Box<dyn Guard> { Box::new(guard) })
                    .collect::<Vec<_>>(),
            )
        };

        let rdef = if config.is_root() {
            ResourceDef::root_prefix(&self.mount_path)
        } else {
            ResourceDef::prefix(&self.mount_path)
        };

        config.register_service(rdef, guards, self, None)
    }
}

impl ServiceFactory<ServiceRequest> for FastCGI {
    type Response = ServiceResponse;
    type Error = Error;
    type Config = ();
    type Service = FastCGIService;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: ()) -> Self::Future {
        let inner = FastCGIInner {
            root: self.root.clone(),
            fastcgi_address: self.fastcgi_address.clone(),
        };
        Box::pin(async move { Ok(FastCGIService(Rc::new(inner))) })
    }
}
