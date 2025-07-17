# `actix-revproxy`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/actix-revproxy?label=latest)](https://crates.io/crates/actix-revproxy)
[![Documentation](https://docs.rs/actix-revproxy/badge.svg?version=0.1.0)](https://docs.rs/actix-revproxy/0.1.0)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/actix-revproxy.svg)
<br />
[![dependency status](https://deps.rs/crate/actix-revproxy/0.1.0/status.svg)](https://deps.rs/crate/actix-revproxy/0.1.0)
[![Download](https://img.shields.io/crates/d/actix-revproxy.svg)](https://crates.io/crates/actix-revproxy)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

Configurable Reverse Proxy service for Actix Web.

Provides a non-blocking service for accessing web-content as a client.

## Examples

```rust
use actix_web::App;
use actix_fastcgi::RevProxy;

let app = App::new()
  .service(FastCGI::new("/", "http://example.com"));
```

<!-- cargo-rdme end -->
