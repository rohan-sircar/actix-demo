use actix_http::body::MessageBody;
use actix_http::header::Header;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::Data;
use actix_web::Error;
use actix_web_httpauth::headers::authorization;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};

use crate::routes::auth::get_claims;
use crate::AppData;

pub struct DomainRootSpanBuilder;

impl RootSpanBuilder for DomainRootSpanBuilder {
    fn on_request_start(req: &ServiceRequest) -> Span {
        let app_data = &req
            .app_data::<Data<AppData>>()
            .cloned()
            .expect("AppData not initialized");
        let jwt_key = &app_data.jwt_key;
        let claims =
            authorization::Authorization::<authorization::Bearer>::parse(req)
                .map(|auth| auth.into_scheme())
                .ok()
                .and_then(|b| get_claims(jwt_key, b.token()).ok());

        let auth_user_id = claims.map(|c| c.custom.user_id.as_uint());
        tracing_actix_web::root_span!(req, auth_user_id,)
    }

    fn on_request_end<B: MessageBody>(
        span: Span,
        outcome: &Result<ServiceResponse<B>, Error>,
    ) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
