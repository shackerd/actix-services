use actix_authn::basic::Basic;

mod common;

#[test]
fn test_basic() {
    common::setup();

    let basic = Basic::default()
        .passwd("admin:$5$zf2X2LFe6AL0ZBWn$y9Ox4HNZwHtZM85cNW8iaUgE7EiuNF01vNveiXnwk68")
        .passwd("root:$5$vLlpnMfbVd1XiSRD$rvDzQkFjgGBdPux.2vt7Z0URtJDubQVWA2f8weMo5XC");
    assert!(basic.verify("admin", "admin"));
    assert!(!basic.verify("admin", "wrong"));
    assert!(basic.verify("root", "password"));
    assert!(!basic.verify("root", "wrong"));
}
