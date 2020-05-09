use actix_web_httpauth::extractors::basic::BasicAuth;

// use actix_identity::Identity;
use actix_web::{dev::ServiceRequest, Error};

pub async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, Error> {
    println!("{}", credentials.user_id());
    println!("{:?}", credentials.password());
    // verify credentials from db
    Ok(req)
}
