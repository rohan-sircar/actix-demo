use std::time::Duration;

use actix_extensible_rate_limit::backend::SimpleInput;
use actix_extensible_rate_limit::HeaderCompatibleOutput;
use actix_extensible_rate_limit::{
    backend::{redis::RedisBackend, SimpleInputFunctionBuilder, SimpleOutput},
    RateLimiter,
};
use actix_http::header::{HeaderName, HeaderValue, RETRY_AFTER};
use actix_web::HttpResponse;
use rand::distr::Alphanumeric;
use rand::Rng;

use crate::models::rate_limit::KeyStrategy;
use crate::utils::{self, RateLimitBackend};
use crate::AppData;

#[allow(clippy::declare_interior_mutable_const)]
pub const X_RATELIMIT_LIMIT: HeaderName =
    HeaderName::from_static("x-ratelimit-limit");
#[allow(clippy::declare_interior_mutable_const)]
pub const X_RATELIMIT_REMAINING: HeaderName =
    HeaderName::from_static("x-ratelimit-remaining");
#[allow(clippy::declare_interior_mutable_const)]
pub const X_RATELIMIT_RESET: HeaderName =
    HeaderName::from_static("x-ratelimit-reset");

fn build_input_function(
    app_data: &AppData,
    input_fn_builder: SimpleInputFunctionBuilder,
) -> SimpleInputFunctionBuilder {
    match app_data.config.rate_limit.key_strategy {
        KeyStrategy::Ip => input_fn_builder.real_ip_key(),
        KeyStrategy::Random => {
            let random_suffix: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();
            let unique_key = format!("{}-{}", "test", random_suffix);
            input_fn_builder.custom_key(&unique_key)
        }
    }
}

pub fn make_denied_response(status: &SimpleOutput) -> HttpResponse {
    let mut response = HttpResponse::TooManyRequests().finish();
    let map = response.headers_mut();
    map.insert(X_RATELIMIT_LIMIT, HeaderValue::from(status.limit()));
    map.insert(X_RATELIMIT_REMAINING, HeaderValue::from(status.remaining()));
    let seconds: u64 = status.seconds_until_reset();
    map.insert(X_RATELIMIT_RESET, HeaderValue::from(seconds));
    map.insert(RETRY_AFTER, HeaderValue::from(seconds));
    response
}

pub fn create_login_rate_limiter(
    app_data: &AppData,
) -> RateLimiter<
    RateLimitBackend,
    SimpleOutput,
    impl Fn(
        &actix_web::dev::ServiceRequest,
    ) -> std::future::Ready<Result<SimpleInput, actix_web::Error>>,
> {
    let input_fn_builder = SimpleInputFunctionBuilder::new(
        std::time::Duration::from_secs(
            app_data.config.rate_limit.auth.window_secs,
        ),
        app_data.config.rate_limit.auth.max_requests.into(),
    );
    let input_fn = build_input_function(&app_data, input_fn_builder).build();

    let backend = if app_data.config.rate_limit.disable {
        utils::RateLimitBackend::Noop
    } else {
        let redis_cm = app_data
            .get_redis_conn()
            .expect("Redis connection required for rate limiting");
        utils::RateLimitBackend::Redis(RedisBackend::builder(redis_cm).build())
    };

    let limiter = RateLimiter::builder(backend, input_fn)
        .rollback_condition(Some(|status| {
            status != actix_web::http::StatusCode::UNAUTHORIZED
        }))
        .add_headers()
        .request_denied_response(|status| {
            let _ = tracing::warn!("Reached rate limit for login");
            make_denied_response(status)
        })
        .build();

    limiter
}

pub fn create_api_rate_limiter(
    app_data: &AppData,
) -> RateLimiter<
    RateLimitBackend,
    SimpleOutput,
    impl Fn(
        &actix_web::dev::ServiceRequest,
    ) -> std::future::Ready<Result<SimpleInput, actix_web::Error>>,
> {
    let backend = if app_data.config.rate_limit.disable {
        utils::RateLimitBackend::Noop
    } else {
        let redis_cm = app_data
            .get_redis_conn()
            .expect("Redis connection required for rate limiting");
        utils::RateLimitBackend::Redis(RedisBackend::builder(redis_cm).build())
    };

    let input_fn_builder = SimpleInputFunctionBuilder::new(
        Duration::from_secs(app_data.config.rate_limit.api.window_secs),
        app_data.config.rate_limit.api.max_requests.into(),
    );
    let input_fn = build_input_function(&app_data, input_fn_builder).build();
    RateLimiter::builder(backend, input_fn)
        .add_headers()
        .request_denied_response(|status| {
            let _ = tracing::warn!("Reached rate limit for api");
            make_denied_response(status)
        })
        .build()
}
