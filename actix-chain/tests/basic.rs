use actix_chain::{Chain, Link, next::IsStatus};
use actix_web::{
    App, HttpRequest, HttpResponse, Responder,
    http::StatusCode,
    test::{self, TestRequest},
    web,
};

mod common;

async fn might_fail(req: HttpRequest) -> impl Responder {
    if !req.headers().contains_key("Required-Header") {
        return HttpResponse::NotFound().body("Request Failed");
    }
    HttpResponse::Ok().body("It worked!")
}

async fn default() -> &'static str {
    "First link failed!"
}

#[actix_web::test]
async fn test_basic() {
    common::setup();

    let srv = test::init_service(
        App::new().service(
            Chain::default()
                .link(Link::new(web::get().to(might_fail)))
                .link(Link::new(web::get().to(default))),
        ),
    )
    .await;

    let req = TestRequest::with_uri("/").to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(common::get_body(res).await, "First link failed!");

    let req = TestRequest::with_uri("/")
        .insert_header(("Required-Header", "value"))
        .to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(common::get_body(res).await, "It worked!");
}

#[actix_web::test]
async fn test_prefix() {
    common::setup();

    let srv = test::init_service(
        App::new().service(
            Chain::default()
                .link(Link::new(web::get().to(might_fail)).prefix("/unstable/"))
                .link(Link::new(web::get().to(default))),
        ),
    )
    .await;

    let req = TestRequest::with_uri("/")
        .insert_header(("Required-Header", "value"))
        .to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(common::get_body(res).await, "First link failed!");

    let req = TestRequest::with_uri("/unstable/test")
        .insert_header(("Required-Header", "value"))
        .to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(common::get_body(res).await, "It worked!");

    let req = TestRequest::with_uri("/unstable/test").to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(common::get_body(res).await, "First link failed!");
}

#[actix_web::test]
async fn test_next() {
    common::setup();

    let srv = test::init_service(
        App::new().service(
            Chain::default()
                .link(Link::new(web::get().to(might_fail)).next(IsStatus(StatusCode::OK)))
                .link(Link::new(web::get().to(default))),
        ),
    )
    .await;

    let req = TestRequest::with_uri("/")
        .insert_header(("Required-Header", "value"))
        .to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "200 OK");
    assert_eq!(common::get_body(res).await, "First link failed!");

    let req = TestRequest::with_uri("/").to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "404 Not Found");
    assert_eq!(common::get_body(res).await, "Request Failed");
}
