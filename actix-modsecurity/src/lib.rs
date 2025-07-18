//! Actix-Web Middleware for the [ModSecurity](https://github.com/owasp-modsecurity/ModSecurity/) library.
//!
//! # Example
//!
//! ```
//! use actix_web::App;
//! use actix_modsecurity::ModSecurity;
//!
//! let mut security = ModSecurity::new();
//! security.add_rules(r#"
//!     SecRuleEngine On
//!
//!     SecRule REQUEST_URI "@rx admin" "id:1,phase:1,deny,status:401"
//! "#).expect("Failed to add rules");
//!
//! let app = App::new()
//!   .wrap(security.middleware());
//! ```
//!
//! # Documentation
//!
//! Information regarding the ModSecurity language can be found in the [ModSecurity Reference Manual](https://github.com/owasp-modsecurity/ModSecurity/wiki/Reference-Manual-(v3.x)).
//!
//! Documentation for this crate can be found on [docs.rs](https://docs.rs/actix-modsecurity).
//!
//! # Requirements
//!
//! This crate requires `libmodsecurity` >= 3.0.6 to be installed on your system.
mod error;
mod factory;
mod modsecurity;
mod service;

pub use factory::Middleware;
pub use modsecurity::{Intervention, ModSecurity, Transaction};
pub use service::ModSecurityService;
