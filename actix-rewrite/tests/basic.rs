use std::collections::HashMap;

use actix_http::header::{self, HeaderValue};
use actix_rewrite::Engine;
use actix_web::{
    HttpRequest, HttpResponse, Responder, body, get,
    test::{self, TestRequest},
    web,
};
use serde::{Deserialize, Serialize};

type QueryMap = web::Query<HashMap<String, String>>;

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    path: String,
    query: HashMap<String, String>,
}

#[get("/index.php")]
async fn index(req: HttpRequest, query: QueryMap) -> impl Responder {
    HttpResponse::Ok().json(Response {
        path: req.path().to_string(),
        query: query.into_inner(),
    })
}

#[actix_web::test]
async fn basic_rewrite() {
    let mut engine = Engine::new();
    engine
        .add_rules(
            r#"
        Rewrite /redirect/(.*) /new/$1            [NE,R]
        Rewrite /one/([\w/]*)  /index.php?page=$1 [L]
    "#,
        )
        .expect("failed to load rules");

    let srv = test::init_service(
        actix_web::App::new()
            .wrap(engine.middleware())
            .service(index),
    )
    .await;

    let req = TestRequest::with_uri("/redirect/hello/world").to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "302 Found");
    assert_eq!(
        res.headers().get(header::LOCATION),
        Some(&HeaderValue::from_static("/new/hello/world"))
    );

    let req = TestRequest::with_uri("/one/1/2/3?a=b").to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "200 OK");

    let data = body::to_bytes(res.into_body()).await.unwrap();
    let json: Response = serde_json::from_slice(&data).unwrap();
    assert_eq!(json.path, "/index.php");
    assert_eq!(json.query.len(), 2);
    assert_eq!(json.query.get("a"), Some(&"b".to_string()));
    assert_eq!(json.query.get("page"), Some(&"1/2/3".to_string()));
}
