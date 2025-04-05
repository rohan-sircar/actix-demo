use actix_http::header::HeaderName;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct CustomHeaders {
    timezone: Tz,
}

impl CustomHeaders {
    pub fn new(timezone: Tz) -> Self {
        Self { timezone }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CustomHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CustomHeadersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CustomHeadersMiddleware {
            service,
            timezone: self.timezone,
        }))
    }
}

pub struct CustomHeadersMiddleware<S> {
    service: S,
    timezone: Tz,
}

impl<S, B> Service<ServiceRequest> for CustomHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>,
    >;

    fn poll_ready(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let timezone = self.timezone;
        let fut = Box::pin(self.service.call(req));

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();

            // Set custom date header
            headers.insert(
                HeaderName::from_static("date"),
                timezone
                    .from_utc_datetime(&Utc::now().naive_utc())
                    .format("%Y-%m-%d %H:%M:%S %Z")
                    .to_string()
                    .parse()
                    .unwrap(),
            );

            // Add Vary header
            headers.insert(
                HeaderName::from_static("vary"),
                "Cookie".parse().unwrap(),
            );

            Ok(res)
        })
    }
}
