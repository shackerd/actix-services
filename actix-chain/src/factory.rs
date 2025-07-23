use std::rc::Rc;

use actix_service::ServiceFactory;
use actix_web::{
    Error,
    dev::{AppService, HttpServiceFactory, ResourceDef, ServiceRequest, ServiceResponse},
    guard::Guard,
};
use futures_core::future::LocalBoxFuture;

use crate::link::Link;

use super::service::{ChainInner, ChainService};

/// Actix-Web service chaining service.
///
/// The chain is constructed from a series of [`Link`](crate::Link)
/// instances which encode when services should be run and when
/// their responses should be reguarded in favor of running the next
/// service.
///
/// `Chain` service must be registered with `App::service()` method.
///
/// # Examples
///
/// ```
/// use actix_web::{App, HttpRequest, HttpResponse, Responder, web};
/// use actix_chain::{Chain, Link};
///
/// async fn might_fail(req: HttpRequest) -> impl Responder {
///     if !req.headers().contains_key("Required-Header") {
///         return HttpResponse::NotFound().body("Request Failed");
///     }
///     HttpResponse::Ok().body("It worked!")
/// }
///
/// async fn default() -> &'static str {
///     "First link failed!"
/// }
///
/// App::new().service(
///     Chain::default()
///         .link(Link::new(web::get().to(might_fail)))
///         .link(Link::new(web::get().to(default)))
/// );
/// ```
#[derive(Clone)]
pub struct Chain {
    mount_path: String,
    links: Vec<Link>,
    guards: Vec<Rc<dyn Guard>>,
    body_buffer_size: usize,
}

impl Chain {
    /// Creates new `Chain` instance.
    ///
    /// The first argument (`mount_path`) is the root URL at which the static files are served.
    /// For example, `/assets` will serve files at `example.com/assets/...`.
    pub fn new(mount_path: &str) -> Self {
        Self {
            mount_path: mount_path.to_owned(),
            links: Vec::new(),
            guards: Vec::new(),
            body_buffer_size: 32 * 1024, // 32 kb default
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
    /// use actix_chain::Chain;
    ///
    /// App::new().service(
    ///     Chain::default()
    ///         .guard(Header("Host", "example.com"))
    /// );
    /// ```
    pub fn guard<G: Guard + 'static>(mut self, guards: G) -> Self {
        self.guards.push(Rc::new(guards));
        self
    }

    /// Add a new [`Link`] to the established chain.
    #[inline]
    pub fn link(mut self, link: Link) -> Self {
        self.push_link(link);
        self
    }

    /// Append a [`Link`] via mutable reference for dynamic assignment.
    #[inline]
    pub fn push_link(&mut self, link: Link) -> &mut Self {
        self.links.push(link);
        self
    }
}

impl Default for Chain {
    #[inline]
    fn default() -> Self {
        Self::new("")
    }
}

impl HttpServiceFactory for Chain {
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

impl ServiceFactory<ServiceRequest> for Chain {
    type Response = ServiceResponse;
    type Error = Error;
    type Config = ();
    type Service = ChainService;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: ()) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            let mut links = vec![];
            for link in this.links {
                match link.into_inner().await {
                    Ok(link) => links.push(link),
                    Err(_) => return Err(()),
                }
            }
            Ok(ChainService(Rc::new(ChainInner {
                links,
                body_buffer_size: this.body_buffer_size,
            })))
        })
    }
}
