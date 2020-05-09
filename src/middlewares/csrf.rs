// //! A filter for cross-site request forgery (CSRF).
// //!
// //! This middleware is stateless and [based on request
// //! headers](https://www.owasp.org/index.php/Cross-Site_Request_Forgery_(CSRF)_Prevention_Cheat_Sheet#Verifying_Same_Origin_with_Standard_Headers).
// //!
// //! By default requests are allowed only if one of these is true:
// //!
// //! * The request method is safe (`GET`, `HEAD`, `OPTIONS`). It is the
// //!   applications responsibility to ensure these methods cannot be used to
// //!   execute unwanted actions. Note that upgrade requests for websockets are
// //!   also considered safe.
// //! * The `Origin` header (added automatically by the browser) matches one
// //!   of the allowed origins.
// //! * There is no `Origin` header but the `Referer` header matches one of
// //!   the allowed origins.
// //!
// //! Use [`CsrfFilter::allow_xhr()`](struct.CsrfFilter.html#method.allow_xhr)
// //! if you want to allow requests with unprotected methods via
// //! [CORS](../cors/struct.Cors.html).
// //!
// //! # Example
// //!
// //! ```
// //! # extern crate actix_web;
// //! use actix_web::middleware::csrf;
// //! use actix_web::{http, App, HttpRequest, HttpResponse};
// //!
// //! fn handle_post(_: &HttpRequest) -> &'static str {
// //!     "This action should only be triggered with requests from the same site"
// //! }
// //!
// //! fn main() {
// //!     let app = App::new()
// //!         .middleware(
// //!             csrf::CsrfFilter::new().allowed_origin("https://www.example.com"),
// //!         )
// //!         .resource("/", |r| {
// //!             r.method(http::Method::GET).f(|_| HttpResponse::Ok());
// //!             r.method(http::Method::POST).f(handle_post);
// //!         })
// //!         .finish();
// //! }
// //! ```
// //!
// //! In this example the entire application is protected from CSRF.

// use std::borrow::Cow;
// use std::collections::HashSet;

// use bytes::Bytes;
// use error::{ResponseError, Result};
// use http::{header, HeaderMap, HttpTryFrom, Uri};
// use httprequest::HttpRequest;
// use httpresponse::HttpResponse;
// use middleware::{Middleware, Started};
// use server::Request;

// /// Potential cross-site request forgery detected.
// #[derive(Debug, Fail)]
// pub enum CsrfError {
//     /// The HTTP request header `Origin` was required but not provided.
//     #[fail(display = "Origin header required")]
//     MissingOrigin,
//     /// The HTTP request header `Origin` could not be parsed correctly.
//     #[fail(display = "Could not parse Origin header")]
//     BadOrigin,
//     /// The cross-site request was denied.
//     #[fail(display = "Cross-site request denied")]
//     CsrDenied,
// }

// impl ResponseError for CsrfError {
//     fn error_response(&self) -> HttpResponse {
//         HttpResponse::Forbidden().body(self.to_string())
//     }
// }

// fn uri_origin(uri: &Uri) -> Option<String> {
//     match (
//         uri.scheme_part(),
//         uri.host(),
//         uri.port_part().map(|port| port.as_u16()),
//     ) {
//         (Some(scheme), Some(host), Some(port)) => Some(format!("{}://{}:{}", scheme, host, port)),
//         (Some(scheme), Some(host), None) => Some(format!("{}://{}", scheme, host)),
//         _ => None,
//     }
// }

// fn origin(headers: &HeaderMap) -> Option<Result<Cow<str>, CsrfError>> {
//     headers
//         .get(header::ORIGIN)
//         .map(|origin| {
//             origin
//                 .to_str()
//                 .map_err(|_| CsrfError::BadOrigin)
//                 .map(|o| o.into())
//         })
//         .or_else(|| {
//             headers.get(header::REFERER).map(|referer| {
//                 Uri::try_from(Bytes::from(referer.as_bytes()))
//                     .ok()
//                     .as_ref()
//                     .and_then(uri_origin)
//                     .ok_or(CsrfError::BadOrigin)
//                     .map(|o| o.into())
//             })
//         })
// }

// /// A middleware that filters cross-site requests.
// ///
// /// To construct a CSRF filter:
// ///
// /// 1. Call [`CsrfFilter::build`](struct.CsrfFilter.html#method.build) to
// ///    start building.
// /// 2. [Add](struct.CsrfFilterBuilder.html#method.allowed_origin) allowed
// ///    origins.
// /// 3. Call [finish](struct.CsrfFilterBuilder.html#method.finish) to retrieve
// ///    the constructed filter.
// ///
// /// # Example
// ///
// /// ```
// /// use actix_web::middleware::csrf;
// /// use actix_web::App;
// ///
// /// # fn main() {
// /// let app = App::new()
// ///     .middleware(csrf::CsrfFilter::new().allowed_origin("https://www.example.com"));
// /// # }
// /// ```
// #[derive(Default)]
// pub struct CsrfFilter {
//     origins: HashSet<String>,
//     allow_xhr: bool,
//     allow_missing_origin: bool,
//     allow_upgrade: bool,
// }

// impl CsrfFilter {
//     /// Start building a `CsrfFilter`.
//     pub fn new() -> CsrfFilter {
//         CsrfFilter {
//             origins: HashSet::new(),
//             allow_xhr: false,
//             allow_missing_origin: false,
//             allow_upgrade: false,
//         }
//     }

//     /// Add an origin that is allowed to make requests. Will be verified
//     /// against the `Origin` request header.
//     pub fn allowed_origin<T: Into<String>>(mut self, origin: T) -> CsrfFilter {
//         self.origins.insert(origin.into());
//         self
//     }

//     /// Allow all requests with an `X-Requested-With` header.
//     ///
//     /// A cross-site attacker should not be able to send requests with custom
//     /// headers unless a CORS policy whitelists them. Therefore it should be
//     /// safe to allow requests with an `X-Requested-With` header (added
//     /// automatically by many JavaScript libraries).
//     ///
//     /// This is disabled by default, because in Safari it is possible to
//     /// circumvent this using redirects and Flash.
//     ///
//     /// Use this method to enable more lax filtering.
//     pub fn allow_xhr(mut self) -> CsrfFilter {
//         self.allow_xhr = true;
//         self
//     }

//     /// Allow requests if the expected `Origin` header is missing (and
//     /// there is no `Referer` to fall back on).
//     ///
//     /// The filter is conservative by default, but it should be safe to allow
//     /// missing `Origin` headers because a cross-site attacker cannot prevent
//     /// the browser from sending `Origin` on unprotected requests.
//     pub fn allow_missing_origin(mut self) -> CsrfFilter {
//         self.allow_missing_origin = true;
//         self
//     }

//     /// Allow cross-site upgrade requests (for example to open a WebSocket).
//     pub fn allow_upgrade(mut self) -> CsrfFilter {
//         self.allow_upgrade = true;
//         self
//     }

//     fn validate(&self, req: &Request) -> Result<(), CsrfError> {
//         let is_upgrade = req.headers().contains_key(header::UPGRADE);
//         let is_safe = req.method().is_safe() && (self.allow_upgrade || !is_upgrade);

//         if is_safe || (self.allow_xhr && req.headers().contains_key("x-requested-with")) {
//             Ok(())
//         } else if let Some(header) = origin(req.headers()) {
//             match header {
//                 Ok(ref origin) if self.origins.contains(origin.as_ref()) => Ok(()),
//                 Ok(_) => Err(CsrfError::CsrDenied),
//                 Err(err) => Err(err),
//             }
//         } else if self.allow_missing_origin {
//             Ok(())
//         } else {
//             Err(CsrfError::MissingOrigin)
//         }
//     }
// }

// impl<S> Middleware<S> for CsrfFilter {
//     fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
//         self.validate(req)?;
//         Ok(Started::Done)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use http::Method;
//     use test::TestRequest;

//     #[test]
//     fn test_safe() {
//         let csrf = CsrfFilter::new().allowed_origin("https://www.example.com");

//         let req = TestRequest::with_header("Origin", "https://www.w3.org")
//             .method(Method::HEAD)
//             .finish();

//         assert!(csrf.start(&req).is_ok());
//     }

//     #[test]
//     fn test_csrf() {
//         let csrf = CsrfFilter::new().allowed_origin("https://www.example.com");

//         let req = TestRequest::with_header("Origin", "https://www.w3.org")
//             .method(Method::POST)
//             .finish();

//         assert!(csrf.start(&req).is_err());
//     }

//     #[test]
//     fn test_referer() {
//         let csrf = CsrfFilter::new().allowed_origin("https://www.example.com");

//         let req =
//             TestRequest::with_header("Referer", "https://www.example.com/some/path?query=param")
//                 .method(Method::POST)
//                 .finish();

//         assert!(csrf.start(&req).is_ok());
//     }

//     #[test]
//     fn test_upgrade() {
//         let strict_csrf = CsrfFilter::new().allowed_origin("https://www.example.com");

//         let lax_csrf = CsrfFilter::new()
//             .allowed_origin("https://www.example.com")
//             .allow_upgrade();

//         let req = TestRequest::with_header("Origin", "https://cswsh.com")
//             .header("Connection", "Upgrade")
//             .header("Upgrade", "websocket")
//             .method(Method::GET)
//             .finish();

//         assert!(strict_csrf.start(&req).is_err());
//         assert!(lax_csrf.start(&req).is_ok());
//     }
// }
