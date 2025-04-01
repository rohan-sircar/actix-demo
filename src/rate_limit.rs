use std::time::Duration;

use actix_extensible_rate_limit::backend::SimpleInput;
use actix_extensible_rate_limit::HeaderCompatibleOutput;
use actix_extensible_rate_limit::{
    backend::{
        memory::InMemoryBackend, redis::RedisBackend,
        SimpleInputFunctionBuilder, SimpleOutput,
    },
    RateLimiter,
};
use actix_http::header::{HeaderName, HeaderValue, RETRY_AFTER};
use actix_web::HttpResponse;
use rand::distr::Alphanumeric;
use rand::Rng;

use crate::models::rate_limit::{
    KeyStrategy, RateLimitConfig, RateLimitPolicy,
};
use crate::utils;
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
    key_strategy: &KeyStrategy,
    input_fn_builder: SimpleInputFunctionBuilder,
) -> SimpleInputFunctionBuilder {
    match key_strategy {
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

pub fn initialize_rate_limit_backend(
    app_data: &AppData,
) -> utils::RateLimitBackend {
    if app_data.config.rate_limit.disable {
        utils::RateLimitBackend::Noop
    } else {
        let redis_cm = app_data.redis_conn_manager.clone();
        utils::RateLimitBackend::Redis(RedisBackend::builder(redis_cm).build())
    }
}

pub fn initialize_hc_backend(enabled: bool) -> utils::RateLimitBackend {
    if !enabled {
        utils::RateLimitBackend::Noop
    } else {
        utils::RateLimitBackend::InMemory(InMemoryBackend::builder().build())
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
    config: &RateLimitConfig,
    backend: utils::RateLimitBackend,
) -> RateLimiter<
    utils::RateLimitBackend,
    SimpleOutput,
    impl Fn(
        &actix_web::dev::ServiceRequest,
    ) -> std::future::Ready<Result<SimpleInput, actix_web::Error>>,
> {
    let input_fn_builder = SimpleInputFunctionBuilder::new(
        std::time::Duration::from_secs(config.auth.window_secs),
        config.auth.max_requests.into(),
    );
    let input_fn =
        build_input_function(&config.key_strategy, input_fn_builder).build();

    RateLimiter::builder(backend, input_fn)
        .rollback_condition(Some(|status| {
            status != actix_web::http::StatusCode::UNAUTHORIZED
        }))
        .add_headers()
        .request_denied_response(|status| {
            let _ = tracing::warn!("Reached rate limit for login");
            make_denied_response(status)
        })
        .build()
}

pub fn create_api_rate_limiter(
    key_strategy: &KeyStrategy,
    policy: &RateLimitPolicy,
    backend: utils::RateLimitBackend,
) -> RateLimiter<
    utils::RateLimitBackend,
    SimpleOutput,
    impl Fn(
        &actix_web::dev::ServiceRequest,
    ) -> std::future::Ready<Result<SimpleInput, actix_web::Error>>,
> {
    let input_fn_builder = SimpleInputFunctionBuilder::new(
        Duration::from_secs(policy.window_secs),
        policy.max_requests.into(),
    );
    let input_fn = build_input_function(key_strategy, input_fn_builder).build();
    RateLimiter::builder(backend, input_fn)
        .add_headers()
        .request_denied_response(|status| {
            let _ = tracing::warn!("Reached rate limit for api");
            make_denied_response(status)
        })
        .build()
}

pub fn create_hc_rate_limiter(
    config: &RateLimitConfig,
    backend: utils::RateLimitBackend,
) -> RateLimiter<
    utils::RateLimitBackend,
    SimpleOutput,
    impl Fn(
        &actix_web::dev::ServiceRequest,
    ) -> std::future::Ready<Result<SimpleInput, actix_web::Error>>,
> {
    let input_fn_builder = SimpleInputFunctionBuilder::new(
        Duration::from_secs(config.api_public.window_secs),
        config.api_public.max_requests.into(),
    );
    let input_fn =
        build_input_function(&config.key_strategy, input_fn_builder).build();

    RateLimiter::builder(backend, input_fn)
        .add_headers()
        .request_denied_response(|status| {
            let _ = tracing::warn!("Reached rate limit for healthcheck");
            make_denied_response(status)
        })
        .build()
}
