# `actix-chain`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/actix-chain?label=latest)](https://crates.io/crates/actix-chain)
[![Documentation](https://docs.rs/actix-chain/badge.svg?version=0.1.0)](https://docs.rs/actix-chain/0.1.0)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/actix-chain.svg)
<br />
[![dependency status](https://deps.rs/crate/actix-chain/0.1.0/status.svg)](https://deps.rs/crate/actix-chain/0.1.0)
[![Download](https://img.shields.io/crates/d/actix-chain.svg)](https://crates.io/crates/actix-chain)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

Actix-Web service chaining service.

Provides a simple non-blocking service for chaining together other arbitrary services.

## Examples

```rust
use actix_web::{App, HttpRequest, HttpResponse, Responder, web};
use actix_chain::{Chain, Link};

async fn might_fail(req: HttpRequest) -> impl Responder {
  if !req.headers().contains_key("Required-Header") {
    return HttpResponse::NotFound().body("Request Failed");
  }
  HttpResponse::Ok().body("It worked!")
}

async fn default() -> &'static str {
  "First link failed!"
}

App::new().service(
  Chain::default()
    .link(Link::new(web::get().to(might_fail)))
    .link(Link::new(web::get().to(default)))
);
```

<!-- cargo-rdme end -->
