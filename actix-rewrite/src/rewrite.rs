//! Utilities for Actix-Web Rewrite Actions

use actix_http::{StatusCode, Uri};
use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse};
use mod_rewrite::context::{EngineCtx, ServerCtx};

use crate::Middleware;

use super::error::Error;
use super::util;

/// Actix-Web compatible wrapper on [`Rewrite`](mod_rewrite::Rewrite)
pub enum Rewrite {
    Uri(Uri),
    Redirect(HttpResponse),
    Response(HttpResponse),
}

/// Actix-Web compatible wrapper on [`Engine`](mod_rewrite::Engine)
pub struct Engine {
    engine: mod_rewrite::Engine,
    srv_ctx: ServerCtx,
}

impl Engine {
    /// Creates a new [`Engine`](crate::Engine) instance.
    ///
    /// See [`mod_rewrite::Engine`](mod_rewrite::Engine) for more details.
    pub fn new() -> Self {
        Self {
            engine: mod_rewrite::Engine::default(),
            srv_ctx: ServerCtx::default(),
        }
    }

    /// Configure max number of loops over entire ruleset during
    /// rewrite before error.
    ///
    /// See [`mod_rewrite::Engine::max_iterations`](mod_rewrite::Engine::max_iterations)
    /// for more details.
    pub fn max_iterations(mut self, iterations: usize) -> Self {
        self.engine = self.engine.max_iterations(iterations);
        self
    }

    /// Pass a configured [`ServerCtx`](crate::ServerCtx) instance
    /// to the engine to use when running [`Engine::rewrite`]
    pub fn server_context(mut self, ctx: ServerCtx) -> Self {
        self.srv_ctx = ctx;
        self
    }

    /// Parses additonal rewrite expressions to append to the engine.
    ///
    /// See [`mod_rewrite::Engine::add_rules`](mod_rewrite::Engine::add_rules)
    /// for more details.
    pub fn add_rules(&mut self, rules: &str) -> Result<&mut Self, Error> {
        self.engine.add_rules(rules)?;
        Ok(self)
    }

    /// Evaluates the given [`HttpRequest`](actix_web::HttpRequest) against
    /// the engine rules and returns a [`Rewrite`] response.
    pub fn rewrite(&self, req: &HttpRequest) -> Result<Rewrite, Error> {
        let mut ctx = EngineCtx::default()
            .with_env()
            .with_time()
            .with_ctx(util::request_ctx(req))
            .with_ctx(self.srv_ctx.clone());
        Ok(
            match self.engine.rewrite_ctx(&req.uri().to_string(), &mut ctx)? {
                mod_rewrite::Rewrite::Uri(uri) => Rewrite::Uri(util::recode(uri)?),
                mod_rewrite::Rewrite::EndUri(uri) => Rewrite::Uri(util::recode(uri)?),
                mod_rewrite::Rewrite::Redirect(uri, sc) => Rewrite::Redirect(
                    HttpResponse::build(StatusCode::from_u16(sc)?)
                        .insert_header((header::LOCATION, uri))
                        .body(""),
                ),
                mod_rewrite::Rewrite::StatusCode(sc) => {
                    Rewrite::Response(HttpResponse::new(StatusCode::from_u16(sc)?))
                }
            },
        )
    }

    /// Converts Engine Instance into Actix-Web Middleware
    ///
    /// # Examples
    ///
    /// ```
    /// use actix_web::App;
    /// use actix_rewrite::Engine;
    ///
    /// let mut engine = Engine::new();
    /// engine.add_rules("RewriteEngine On\n").expect("Failed to add rules");
    ///
    /// let app = App::new()
    ///     .wrap(engine.middleware());
    /// ```
    #[inline]
    pub fn middleware(self) -> Middleware {
        self.into()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
