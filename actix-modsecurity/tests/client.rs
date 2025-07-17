use actix_modsecurity::{Middleware, ModSecurity};
use actix_web::test::{self, TestRequest};

mod common;

const RULES: &str = r#"
SecRuleEngine On

SecRule REQUEST_URI "@rx admin" "id:1,phase:1,deny,status:401"

SecRule REQUEST_HEADERS:X-Client-Port "@streq 22" \
    "id:'1234567',\
    log,\
    msg:'Blocking SSH port',\
    phase:1,\
    t:none,\
    status:403,\
    deny
"#;

#[actix_web::test]
async fn test_middleware() {
    common::setup();

    let mut security = ModSecurity::new();
    security.add_rules(RULES).expect("Failed to add rules");

    let mw: Middleware = security.into();
    let srv = test::init_service(actix_web::App::new().wrap(mw)).await;

    let req = TestRequest::with_uri("/").to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "404 Not Found");

    let req = TestRequest::with_uri("/admin").to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "401 Unauthorized");

    let req = TestRequest::with_uri("/")
        .insert_header(("X-Client-Port", 22))
        .to_request();
    let res = test::call_service(&srv, req).await;
    assert_eq!(res.status().to_string(), "403 Forbidden");
}
