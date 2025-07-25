# `actix-sanitize`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/actix-sanitize?label=latest)](https://crates.io/crates/actix-sanitize)
[![Documentation](https://docs.rs/actix-sanitize/badge.svg?version=0.1.0)](https://docs.rs/actix-sanitize/0.1.0)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/actix-sanitize.svg)
<br />
[![dependency status](https://deps.rs/crate/actix-sanitize/0.1.0/status.svg)](https://deps.rs/crate/actix-sanitize/0.1.0)
[![Download](https://img.shields.io/crates/d/actix-sanitize.svg)](https://crates.io/crates/actix-sanitize)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

Actix-Web Middleware for "Sanitising" unwanted error messages.

## Examples

```rust
use actix_web::{App, error, Responder, http::StatusCode, web};
use actix_sanitize::Sanitizer;

async fn faulty_service() -> impl Responder {
    error::InternalError::new(
        "sensitive error message",
        StatusCode::INTERNAL_SERVER_ERROR,
    )
}

let app = App::new()
    .wrap(Sanitizer::default())
    .service(
        web::resource("/broken")
            .route(web::get().to(faulty_service))
    );
```

<!-- cargo-rdme end -->
