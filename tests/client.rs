//! Basic FastCGI Tests

use std::collections::HashMap;

use actix_web::{
    body,
    http::header,
    test::{self, TestRequest},
};

mod common;
use common::*;
use http::{HeaderValue, Method};

#[actix_web::test]
async fn test_simple_get() {
    setup();

    let srv = spawn_test_server!();
    let req = TestRequest::with_uri("/hello.php").to_request();
    let res = test::call_service(&srv, req).await;

    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(
        res.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("text/plain; charset=utf-8"))
    );

    let content = res.into_body();
    let data = body::to_bytes(content).await.expect("missing body");
    let body = std::str::from_utf8(&data).expect("invalid body");
    assert_eq!(body, "Hello World!");
}

#[actix_web::test]
async fn test_simple_post() {
    setup();

    let mut form = HashMap::new();
    form.insert("test", "World!");

    let srv = spawn_test_server!();
    let req = TestRequest::with_uri("/post.php")
        .method(Method::POST)
        .append_header(("X-TEST", "Hello"))
        .set_form(form)
        .append_header(("Content-Length", 13))
        .to_request();
    let res = test::call_service(&srv, req).await;

    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(
        res.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("text/html; charset=UTF-8"))
    );

    let content = res.into_body();
    let data = body::to_bytes(content).await.expect("missing body");
    let body = std::str::from_utf8(&data).expect("invalid body");
    assert_eq!(body, "Hello World!");
}

#[actix_web::test]
async fn test_simple_json() {
    setup();

    let mut json = HashMap::new();
    json.insert("one", "Hello");
    json.insert("two", "World!");

    let srv = spawn_test_server!();
    let req = TestRequest::with_uri("/json.php")
        .method(Method::PUT)
        .set_json(json)
        .append_header(("Content-Length", 30))
        .to_request();
    let res = test::call_service(&srv, req).await;

    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(
        res.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("text/html; charset=UTF-8"))
    );

    let content = res.into_body();
    let data = body::to_bytes(content).await.expect("missing body");
    let body = std::str::from_utf8(&data).expect("invalid body");
    assert_eq!(body, "Hello World!");
}
