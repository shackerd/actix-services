//! Utiliies Used for Actix ModRewrite

use std::{collections::HashMap, str::FromStr};

use actix_http::Uri;
use actix_web::{HttpRequest, web::Query};
use mod_rewrite::context::{RequestCtx, ServerCtx};

use super::error::Error;

type QueryMap = Query<HashMap<String, String>>;

#[inline]
pub(crate) fn recode(uri: String) -> Result<Uri, Error> {
    Ok(Uri::from_str(&uri)?)
}

/// Build [`mod_rewrite::context::RequestCtx`]
/// using [`HttpRequest`](actix_web::HttpRequest) data.
pub fn request_ctx(req: &HttpRequest) -> RequestCtx {
    RequestCtx::default()
        .path_info(req.match_info().unprocessed())
        .request_uri(req.uri().to_string())
        .request_method(req.method().to_string())
        .query_string(req.uri().query().unwrap_or(""))
        .maybe_remote_addr(req.peer_addr())
        .expect("invalid peer address")
}

/// Fill [`mod_rewrite::context::ServerCtx`]
/// using [`HttpRequest`](actix_web::HttpRequest) data.
pub fn fill_server_ctx(ctx: ServerCtx, req: &HttpRequest) -> Result<ServerCtx, Error> {
    Ok(ctx
        .server_addr(req.app_config().local_addr())?
        .server_protocol(if req.app_config().secure() {
            "https"
        } else {
            "http"
        }))
}

#[inline]
fn get_query(uri: &Uri) -> Result<QueryMap, Error> {
    Ok(QueryMap::from_query(uri.query().unwrap_or(""))?)
}

/// Build new URI combining data from [`actix_web::HttpRequest`]
/// and rewritten uri from [`Engine::rewrite`](crate::Engine::rewrite)
#[inline]
pub fn join_uri(before: &Uri, after: &Uri) -> Result<Uri, Error> {
    let mut query = get_query(before)?;
    query.extend(get_query(after)?.into_inner());
    let query = serde_urlencoded::to_string(query.into_inner())?;

    let scheme = after
        .scheme()
        .or(before.scheme())
        .map(|scheme| format!("{}://", scheme.as_str()))
        .unwrap_or_default();
    let authority = after
        .authority()
        .or(before.authority())
        .map(|authority| authority.as_str())
        .unwrap_or_default();
    let path = after.path();

    let uri = format!("{scheme}{authority}{path}?{query}");
    Ok(Uri::from_str(&uri)?)
}
