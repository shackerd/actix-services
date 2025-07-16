///! Common Testing Utilities
use std::{sync::Once};

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

#[allow(unused_macros)]
macro_rules! spawn_test_server {
    () => {{
        let fgi = actix_fastcgi::FastCGI::new("", "tests/php", "127.0.0.1:9000");
        test::init_service(actix_web::App::new().service(fgi)).await
    }};
}

#[allow(unused_imports)]
pub(crate) use spawn_test_server;
