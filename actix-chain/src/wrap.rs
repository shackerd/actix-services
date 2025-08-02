use actix_service::Transform;
use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
};

use crate::service::HttpService;

/// Trait Abstraction for Middlware Compatible Objects
pub trait Wrappable {
    fn wrap_with<M, B>(self, middleware: M) -> Self
    where
        M: Transform<
                HttpService,
                ServiceRequest,
                Response = ServiceResponse<B>,
                Error = Error,
                InitError = (),
            > + 'static,
        B: MessageBody + 'static;
}
