use actix_web_httpauth::extractors::basic::BasicAuth;

use crate::AppConfig;
// use actix_identity::Identity;
use crate::routes::validate_basic_auth;
use actix_threadpool::BlockingError;

use actix_web::{dev::ServiceRequest, web, Error};

// use Response;

pub async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, Error> {
    println!("{}", credentials.user_id());
    println!("{:?}", credentials.password());
    // verify credentials from db
    let credentials2 = credentials.clone();
    // let pool = req.app_data();
    let config = req.app_data::<AppConfig>().expect("Error getting db");
    // .get_ref()
    // .clone();
    // let _config = req
    //     .app_data::<Config>()
    //     .map(|data| data.get_ref().clone())
    //     .unwrap_or_else(Default::default);

    let res = web::block(move || validate_basic_auth(credentials2, &config))
        .await
        .and_then(|valid| {
            if valid {
                debug!("Success");
                Ok(req)
            } else {
                debug!("Failure");
                Err(BlockingError::Error(
                    crate::errors::DomainError::new_password_error(
                        "Wrong password or account does not exist".to_string(),
                    ),
                ))
                // Err(AuthenticationError::from(config))
                // Ok(req)
            }
        });
    let res2: Result<ServiceRequest, Error> = res.map_err(|e| e.into());
    res2
}
