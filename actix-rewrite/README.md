# `actix-rewrite`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/actix-rewrite?label=latest)](https://crates.io/crates/actix-rewrite)
[![Documentation](https://docs.rs/actix-rewrite/badge.svg?version=0.1.0)](https://docs.rs/actix-rewrite/0.1.0)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/actix-rewrite.svg)
<br />
[![dependency status](https://deps.rs/crate/actix-rewrite/0.1.0/status.svg)](https://deps.rs/crate/actix-rewrite/0.1.0)
[![Download](https://img.shields.io/crates/d/actix-rewrite.svg)](https://crates.io/crates/actix-rewrite)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

`mod_rewrite` middleware service for Actix Web.

Provides a non-blocking middleware for dynamic rerouting using a complete
rule based engine.

## Examples

```rust
use actix_web::App;
use actix_rewrite::Engine;

let mut engine = Engine::new();
engine.add_rules(r#"
  RewriteRule /file/(.*)     /tmp/$1      [L]
  RewriteRule /redirect/(.*) /location/$1 [R=302]
  RewriteRule /blocked/(.*)  -            [F]
"#).expect("failed to process rules");

let app = App::new()
  .wrap(engine.middleware());
```

<!-- cargo-rdme end -->
