use std::{rc::Rc, str::FromStr};

use actix_service::{IntoServiceFactory, ServiceFactory, ServiceFactoryExt, Transform, boxed};
use actix_web::{
    Error, HttpResponse,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    guard::{Guard, GuardContext},
    http::{StatusCode, Uri, header, uri::PathAndQuery},
    middleware::Compat,
    mime,
};

use crate::{
    Chain,
    next::{IsStatus, Next},
    service::{HttpNewService, HttpService},
};

/// A single [`Link`] in the greater [`Chain`](crate::Chain)
///
/// Wraps an Actix-Web service factory with details on when the service should
/// be evaluated in the chain and if processing should continue afterwards.
///
/// # Examples
///
/// ```
/// use actix_web::{App, guard::Header, http::StatusCode, web};
/// use actix_chain::{Chain, Link, next::IsStatus};
///
/// async fn index() -> &'static str {
///     "Hello World!"
/// }
///
/// Link::new(web::get().to(index))
///     .prefix("/index")
///     .guard(Header("Host", "example.com"))
///     .next(IsStatus(StatusCode::NOT_FOUND));
/// ```
#[derive(Clone)]
pub struct Link {
    pub(crate) prefix: String,
    pub(crate) guards: Vec<Rc<dyn Guard>>,
    pub(crate) next: Vec<Rc<dyn Next>>,
    service: Rc<HttpNewService>,
}

#[inline]
fn box_factory<F, U>(service: F) -> Rc<HttpNewService>
where
    F: IntoServiceFactory<U, ServiceRequest>,
    U: ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = Error>
        + 'static,
{
    Rc::new(boxed::factory(service.into_factory().map_init_err(|_| ())))
}

impl Link {
    /// Create a new [`Link`] for your [`Chain`](crate::Chain).
    ///
    /// Any Actix-Web service can be passed such as [`actix_web::Route`].
    pub fn new<F, U>(service: F) -> Self
    where
        F: IntoServiceFactory<U, ServiceRequest>,
        U: ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = Error>
            + 'static,
    {
        Self {
            prefix: String::new(),
            guards: Vec::new(),
            next: Vec::new(),
            service: box_factory(service),
        }
    }

    /// Assign a `match-prefix` / `mount_path` to the link.
    ///
    /// The prefix is the root URL at which the service is used.
    /// For example, /assets will serve files at example.com/assets/....
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Adds a routing guard.
    ///
    /// Use this to allow multiple chained services that respond to strictly different
    /// properties of a request.
    ///
    /// **IMPORTANT:** If a guard supplied here does not match a given request,
    /// the request WILL be forwarded to the next [`Link`] in the chain, unlike
    /// [`Chain::guard`](crate::Chain::guard)
    ///
    /// # Examples
    /// ```
    /// use actix_web::{guard::Header, App, web};
    /// use actix_chain::{Chain, Link};
    ///
    /// async fn index() -> &'static str {
    ///     "Hello world!"
    /// }
    ///
    /// let svc = web::get().to(index);
    /// Chain::default()
    ///     .link(Link::new(svc)
    ///         .guard(Header("Host", "example.com")));
    /// ```
    pub fn guard<G: Guard + 'static>(mut self, guards: G) -> Self {
        self.guards.push(Rc::new(guards));
        self
    }

    /// Configure when a [`Link`] should forward to the next chain
    /// instead of returning its [`ServiceResponse`](actix_web::dev::ServiceResponse).
    ///
    /// Any responses that match the supplied criteria will instead be ignored,
    /// assuming another link exists within the chain.
    ///
    /// The default [`Link`] behavior is to continue down the chain
    /// on "404 Not Found" and "405 Method Not Allowed" responses only.
    ///
    /// # Examples
    /// ```
    /// use actix_web::{http::StatusCode, web};
    /// use actix_chain::{Link, next::IsStatus};
    ///
    /// async fn index() -> &'static str {
    ///     "Hello world!"
    /// }
    ///
    /// Link::new(web::get().to(index))
    ///     .next(IsStatus(StatusCode::NOT_FOUND));
    /// ```
    pub fn next<N>(mut self, next: N) -> Self
    where
        N: Next + 'static,
    {
        self.next.push(Rc::new(next));
        self
    }

    /// Registers a link specific middleware.
    ///
    /// Wrapping a link advantagously does not construct
    /// an object of varying type, meaning you can dynamically chain
    /// together middleware during link construction, unlike [`actix_web::App::wrap`]
    ///
    /// See [`actix_web::middleware`] for more details on middleware.
    ///
    /// # Example
    ///
    ///```
    /// use actix_web::{middleware, web, App};
    /// use actix_chain::Link;
    ///
    /// async fn index() -> &'static str {
    ///     "Welcome!"
    /// }
    ///
    /// let link = Link::new(web::get().to(index))
    ///     .wrap(middleware::Logger::default())
    ///     .prefix("/index.html");
    ///```
    #[inline]
    pub fn wrap<M, B>(mut self, middleware: M) -> Self
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
        let svc = actix_service::apply(Compat::new(middleware), self.service.clone());
        self.service = box_factory(svc);
        self
    }

    /// Convert public [`Link`] builder into [`LinkInner`]
    pub(crate) async fn into_inner(&self) -> Result<LinkInner, ()> {
        let guard = match self.guards.is_empty() {
            true => None,
            false => Some(AllGuard(self.guards.clone())),
        };
        let next: Vec<Rc<dyn Next>> = match self.next.is_empty() {
            true => vec![
                IsStatus::rc(StatusCode::NOT_FOUND),
                IsStatus::rc(StatusCode::METHOD_NOT_ALLOWED),
            ],
            false => self.next.clone(),
        };
        Ok(LinkInner {
            guard,
            next,
            prefix: self.prefix.clone(),
            service: Rc::new(self.service.new_service(()).await?),
        })
    }
}

impl From<Chain> for Link {
    fn from(mut value: Chain) -> Self {
        let prefix = value.mount_path.clone();
        let guards: Vec<_> = value.guards.drain(0..).collect();
        let next: Vec<_> = value.next.drain(0..).collect();
        let mut link = Self::new(value).prefix(&prefix);
        link.guards = guards;
        link.next = next;
        link
    }
}

struct AllGuard(Vec<Rc<dyn Guard>>);

impl Guard for AllGuard {
    #[inline]
    fn check(&self, ctx: &actix_web::guard::GuardContext<'_>) -> bool {
        self.0.iter().all(|g| g.check(ctx))
    }
}

/// Default 404 Response when service is unable to respond
#[inline]
pub(crate) fn default_response(req: ServiceRequest) -> ServiceResponse {
    req.into_response(
        HttpResponse::NotFound()
            .insert_header(header::ContentType(mime::TEXT_PLAIN_UTF_8))
            .body("Not Found"),
    )
}

pub(crate) struct LinkInner {
    prefix: String,
    guard: Option<AllGuard>,
    pub(crate) service: Rc<HttpService>,
    pub(crate) next: Vec<Rc<dyn Next>>,
}

impl LinkInner {
    /// Generate new URI with prefix stripped if prefix is not empty
    pub(crate) fn new_uri(&self, uri: &Uri) -> Option<Uri> {
        if self.prefix.is_empty() {
            return None;
        }
        let mut parts = uri.clone().into_parts();
        parts.path_and_query = parts
            .path_and_query
            .and_then(|pq| PathAndQuery::from_str(pq.as_str().strip_prefix(&self.prefix)?).ok());
        Uri::from_parts(parts).ok()
    }

    /// Check if request path matches prefix and any guards are met
    #[inline]
    pub(crate) fn matches(&self, path: &str, ctx: &GuardContext) -> bool {
        path.starts_with(&self.prefix) && self.guard.as_ref().map(|g| !g.check(ctx)).unwrap_or(true)
    }

    /// Check if response is invalid, and next link should execute
    #[inline]
    pub(crate) fn go_next(&self, res: &HttpResponse) -> bool {
        self.next.iter().any(|next| next.next(res))
    }

    /// Call inner service once and return [`actix_web::dev::ServiceResponse`]
    /// no matter what.
    #[inline]
    pub(crate) async fn call_once(
        &self,
        mut req: ServiceRequest,
    ) -> Result<ServiceResponse, Error> {
        if !self.matches(req.uri().path(), &req.guard_ctx()) {
            return Ok(default_response(req));
        }
        if let Some(uri) = self.new_uri(req.uri()) {
            req.head_mut().uri = uri;
        }
        self.service.call(req).await
    }
}
