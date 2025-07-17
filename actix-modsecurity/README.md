# `actix-modsecurity`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/actix-modsecurity?label=latest)](https://crates.io/crates/actix-modsecurity)
[![Documentation](https://docs.rs/actix-modsecurity/badge.svg?version=0.1.0)](https://docs.rs/actix-modsecurity/0.1.0)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/actix-modsecurity.svg)
<br />
[![dependency status](https://deps.rs/crate/actix-modsecurity/0.1.0/status.svg)](https://deps.rs/crate/actix-modsecurity/0.1.0)
[![Download](https://img.shields.io/crates/d/actix-modsecurity.svg)](https://crates.io/crates/actix-modsecurity)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

LibModSecurity middleware service for Actix Web.

Provides a non-blocking middleware for protecting your endpoints with libmodsecurity.

## Examples

```rust
use actix_web::App;
use actix_modsecurity::ModSecurity;

let mut security = ModSecurity::new();
security.add_rules(r#"
    SecRuleEngine On

    SecRule REQUEST_URI "@rx admin" "id:1,phase:1,deny,status:401"
"#).expect("Failed to add rules");

let app = App::new()
  .wrap(security.middleware());
```

<!-- cargo-rdme end -->
