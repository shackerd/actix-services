# `actix-fastcgi`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/actix-fastcgi?label=latest)](https://crates.io/crates/actix-fastcgi)
[![Documentation](https://docs.rs/actix-fastcgi/badge.svg?version=0.1.0)](https://docs.rs/actix-fastcgi/0.1.0)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/actix-fastcgi.svg)
<br />
[![dependency status](https://deps.rs/crate/actix-fastcgi/0.1.0/status.svg)](https://deps.rs/crate/actix-fastcgi/0.1.0)
[![Download](https://img.shields.io/crates/d/actix-fastcgi.svg)](https://crates.io/crates/actix-fastcgi)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

FastCGI Client service for Actix Web.

Provides a non-blocking service for calling fastcgi content as a client.

## Examples

```rust
use actix_web::App;
use actix_fastcgi::FastCGI;

let app = App::new()
  .service(FastCGI::new("/", "/my/php/files", "tcp://127.0.0.1:9000"));
```

<!-- cargo-rdme end -->
