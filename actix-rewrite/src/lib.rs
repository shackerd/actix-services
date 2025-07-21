//! Actix-Web Middleware designed to replicate HTTPd's [`mod_rewrite`](https://httpd.apache.org/docs/current/mod/mod_rewrite.html).
//!
//! # Example
//!
//! ```
//! use actix_web::App;
//! use actix_rewrite::Engine;
//!
//! let mut engine = Engine::new();
//! engine.add_rules(r#"
//!     RewriteRule /file/(.*)     /tmp/$1      [L]
//!     RewriteRule /redirect/(.*) /location/$1 [R=302]
//!     RewriteRule /blocked/(.*)  -            [F]
//! "#).expect("failed to process rules");
//!
//! let app = App::new()
//!   .wrap(engine.middleware());
//! ```
//!
//! # Documentation
//!
//! Information regarding the Rewrite expression language can be found in the [mod_rewrite manual](https://httpd.apache.org/docs/current/mod/mod_rewrite.html).
//!
//! Documentation for this crate can be found on [docs.rs](https://docs.rs/actix-modrewrite).
mod error;
mod factory;
mod rewrite;
mod service;
pub mod util;

pub use error::Error;
pub use factory::Middleware;
pub use rewrite::{Engine, Rewrite};
pub use service::RewriteService;

pub use mod_rewrite::context::ServerCtx;
