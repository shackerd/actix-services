use std::sync::Once;

use actix_web::{
    body::{self, BoxBody},
    dev::ServiceResponse,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static START: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn setup() {
    START.call_once(|| {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}

/// Convert `ServiceResponse` into body content string
pub async fn get_body(res: ServiceResponse<BoxBody>) -> String {
    let content = res.into_body();
    let data = body::to_bytes(content).await.expect("missing body");
    std::str::from_utf8(&data)
        .expect("invalid body")
        .to_string()
}
