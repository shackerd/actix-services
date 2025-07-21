//! Utiliies Used for Actix ModRewrite

use std::collections::HashMap;

use actix_http::{Uri, uri::Scheme};
use actix_web::{HttpRequest, web::Query};
use mod_rewrite::context::RequestCtx;

use super::error::Error;

type QueryMap = Query<HashMap<String, String>>;

/// Build [`mod_rewrite::context::RequestCtx`]
/// using [`ServiceRequest`](actix_web::dev::ServiceRequest) data.
pub fn request_ctx(req: &HttpRequest) -> RequestCtx {
    RequestCtx::default()
        .path_info(req.match_info().unprocessed())
        .request_uri(req.uri().to_string())
        .request_method(req.method().to_string())
        .query_string(req.uri().query().unwrap_or(""))
        .maybe_remote_addr(req.peer_addr())
        .expect("invalid peer address")
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

    let mut builder = Uri::builder().scheme(
        after
            .scheme()
            .or(before.scheme())
            .cloned()
            .unwrap_or(Scheme::HTTP),
    );

    if let Some(authority) = after.authority().or(before.authority()) {
        builder = builder.authority(authority.clone());
    }
    let path = after.path();
    Ok(builder.path_and_query(format!("{path}?{query}")).build()?)
}
