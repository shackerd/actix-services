//! Actix-Web Middleware for "Sanitising" unwanted error messages.
//!
//! # Example
//!
//! ```
//! use actix_web::{App, error, Responder, http::StatusCode, web};
//! use actix_sanitize::Sanitizer;
//!
//! async fn faulty_service() -> impl Responder {
//!     error::InternalError::new(
//!         "sensitive error message",
//!         StatusCode::INTERNAL_SERVER_ERROR,
//!     )
//! }
//!
//! let app = App::new()
//!     .wrap(Sanitizer::default())
//!     .service(
//!         web::resource("/broken")
//!             .route(web::get().to(faulty_service))
//!     );
//! ```
mod error;
mod factory;
pub mod guard;
mod service;

pub use error::SanitizedError;
pub use factory::Sanitizer;
pub use service::SanitizeService;
