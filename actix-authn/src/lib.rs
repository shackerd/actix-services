use actix_web::{Error, HttpRequest, HttpResponse};

mod factory;
mod service;

#[cfg(feature = "basic")]
pub mod basic;

/// Trait abstraction for web authenticator services
pub trait Authenticator {
    fn authorize(&self, req: &HttpRequest) -> impl Future<Output = Result<bool, Error>>;
    fn prompt(&self, req: &HttpRequest) -> impl Future<Output = Result<HttpResponse, Error>>;
}

pub use factory::Authn;
pub use service::AuthnService;
