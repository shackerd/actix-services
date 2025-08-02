use std::rc::Rc;

use actix_service::{ServiceFactory, Transform};
use actix_web::{
    Error,
    body::MessageBody,
    dev::{AppService, HttpServiceFactory, ResourceDef, ServiceRequest, ServiceResponse},
    guard::Guard,
};
use futures_core::future::LocalBoxFuture;

use crate::{link::Link, next::Next, service::HttpService, wrap::Wrappable};

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
    pub(crate) mount_path: String,
    pub(crate) links: Vec<Link>,
    pub(crate) guards: Vec<Rc<dyn Guard>>,
    pub(crate) next: Vec<Rc<dyn Next>>, // For Into<Link> only
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
            next: Vec::new(),
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

    /// Registers a chain specific middleware.
    ///
    /// Wrapping a chain advantagously does not construct an object
    /// of varying type, meaning you can dynamically chain together
    /// middleware during chain construction, unlike [`actix_web::App::wrap`].
    ///
    /// **IMPORTANT:** Since wrapping a chain immediately constructs
    /// a new chain object around its existing contents, [`Chain::wrap`]
    /// should only be called _AFTER_ all links have been added to the
    /// existing chain, rather than before.
    ///
    /// See [`actix_web::middleware`] for more details on middleware.
    ///
    /// # Example
    ///
    /// ```
    /// use actix_web::{middleware, web, App};
    /// use actix_chain::{Chain, Link};
    ///
    /// async fn index() -> &'static str {
    ///     "Welcome!"
    /// }
    ///
    /// let chain = Chain::default()
    ///     .link(Link::new(web::get().to(index)).prefix("/index.html"))
    ///     .wrap(middleware::Logger::default());
    /// ```
    #[inline]
    pub fn wrap<M, B>(self, middleware: M) -> Chain
    where
        M: Transform<
                HttpService,
                ServiceRequest,
                Response = ServiceResponse<B>,
                Error = Error,
                InitError = (),
            > + 'static,
        B: MessageBody + 'static,
    {
        self.wrap_with(middleware)
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

impl Wrappable for Chain {
    /// See [`Chain::wrap`] for more information.
    fn wrap_with<M, B>(mut self, middleware: M) -> Chain
    where
        M: Transform<
                HttpService,
                ServiceRequest,
                Response = ServiceResponse<B>,
                Error = Error,
                InitError = (),
            > + 'static,
        B: MessageBody + 'static,
    {
        let prefix = self.mount_path.clone();
        let guards: Vec<_> = self.guards.drain(0..).collect();
        let next: Vec<_> = self.next.drain(0..).collect();
        let link = Link::from(self).wrap_with(middleware);
        let mut chain = Chain::new(&prefix).link(link);
        chain.next = next;
        chain.guards = guards;
        chain
    }
}

impl Default for Chain {
    #[inline]
    fn default() -> Self {
        Self::new("")
    }
}

impl From<Link> for Chain {
    /// Convert link into single linked chain.
    fn from(mut value: Link) -> Self {
        let prefix = value.prefix.clone();
        let guards: Vec<_> = value.guards.drain(0..).collect();
        let next: Vec<_> = value.next.clone();
        let mut chain = Self::new(&prefix).link(value);
        chain.guards = guards;
        chain.next = next;
        chain
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
        if self.links.is_empty() {
            panic!("Chain contains no links!")
        }
        let this = self.clone();
        Box::pin(async move {
            let mut links = vec![];
            for link in this.links {
                match link.inner().await {
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
