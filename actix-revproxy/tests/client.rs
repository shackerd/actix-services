use std::collections::HashMap;

use actix_revproxy::RevProxy;
use actix_web::{
    http::header::{self, HeaderValue},
    test::{self, TestRequest},
};
use awc::http::Method;
use serde::Deserialize;

mod common;

#[derive(Debug, Deserialize)]
pub struct HttpBinPost {
    args: HashMap<String, String>,
    data: String,
    form: HashMap<String, String>,
    headers: HashMap<String, String>,
    json: Option<HashMap<String, String>>,
    url: String,
}

#[actix_web::test]
async fn simple_get() {
    common::setup();

    let proxy = RevProxy::new("", "http://example.com");
    let srv = test::init_service(actix_web::App::new().service(proxy)).await;

    let req = TestRequest::with_uri("/").to_request();
    let res = test::call_service(&srv, req).await;

    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(
        res.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("text/html"))
    );
    assert_eq!(
        res.headers().get(header::CONTENT_LENGTH),
        Some(&HeaderValue::from_static("1256"))
    );

    let body = common::get_body(res).await;
    assert!(
        body.contains("<title>Example Domain</title>"),
        "invalid body"
    );
}

#[actix_web::test]
async fn simple_post() {
    common::setup();

    let proxy = RevProxy::new("", "http://httpbin.org?hello=world");
    let srv = test::init_service(actix_web::App::new().service(proxy)).await;

    let mut form = HashMap::new();
    form.insert("c", "d");
    form.insert("e", "f");

    let req = TestRequest::with_uri("/post?a=b")
        .method(Method::POST)
        .insert_header(("Test-Header", "helloworld"))
        .insert_header(("Content-Length", 7))
        .set_form(form)
        .to_request();
    let res = test::call_service(&srv, req).await;

    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(
        res.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("application/json"))
    );

    let body = common::get_body(res).await;
    let post: HttpBinPost = serde_json::from_str(&body).expect("invalid json");

    assert!(post.url.starts_with("http://httpbin.org/post?"));
    assert_eq!(post.args.get("hello"), Some(&"world".to_string()));
    assert_eq!(post.args.get("a"), Some(&"b".to_string()));
    assert_eq!(post.data, "");
    assert_eq!(post.form.get("c"), Some(&"d".to_string()));
    assert_eq!(post.form.get("e"), Some(&"f".to_string()));
    assert_eq!(
        post.headers.get("Test-Header"),
        Some(&"helloworld".to_string())
    );
    assert_eq!(post.json, None);
}
