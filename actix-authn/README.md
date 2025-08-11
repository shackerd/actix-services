# `actix-authn`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/actix-authn?label=latest)](https://crates.io/crates/actix-authn)
[![Documentation](https://docs.rs/actix-authn/badge.svg?version=0.1.0)](https://docs.rs/actix-authn/0.1.0)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/actix-authn.svg)
<br />
[![dependency status](https://deps.rs/crate/actix-authn/0.1.0/status.svg)](https://deps.rs/crate/actix-authn/0.1.0)
[![Download](https://img.shields.io/crates/d/actix-authn.svg)](https://crates.io/crates/actix-authn)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

Authentication service helpers for Actix Web.

Provides a non-blocking middleware for securing endpoints with authentication.

## Examples

```rust
use actix_web::App;
use actix_authn::{Authn, basic::{Basic, Passwd}};

/// passwords should be generated outside of HttpServer::new
/// or use [`Basic::passwd`] or [`Basic::htpasswd`].
let passwd = Passwd::new("admin");

let app = App::new()
    .wrap(Authn::new(Basic::default().auth("admin", passwd).build()));
```

<!-- cargo-rdme end -->
