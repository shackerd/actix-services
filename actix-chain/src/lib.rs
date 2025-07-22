//! Actix-Web service chaining service.
//!
//! Provides a simple non-blocking service for chaining together other arbitrary services.
//!
//! # Example
//!
//! ```
//! use actix_web::{App, HttpRequest, HttpResponse, Responder, web};
//! use actix_chain::{Chain, Link};
//!
//! async fn might_fail(req: HttpRequest) -> impl Responder {
//!     if !req.headers().contains_key("Required-Header") {
//!         return HttpResponse::NotFound().body("Request Failed");
//!     }
//!     HttpResponse::Ok().body("It worked!")
//! }
//!
//! async fn default() -> &'static str {
//!     "First link failed!"
//! }
//!
//! App::new().service(
//!     Chain::default()
//!         .link(Link::new(web::get().to(might_fail)))
//!         .link(Link::new(web::get().to(default)))
//! );
//! ```

mod factory;
mod link;
pub mod next;
mod payload;
mod service;

pub use factory::Chain;
pub use link::Link;
pub use service::ChainService;
