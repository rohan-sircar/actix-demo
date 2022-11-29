use actix_web::{HttpResponse, ResponseError};
use bcrypt::BcryptError;
use custom_error::custom_error;
use derive_more::Display;
// use derive_more::Display;
// use diesel::result::DatabaseErrorKind;
use crate::models::api_response::*;
use std::convert::From;

#[derive(Debug, Display)]
pub struct JwtErrorFoo(String);

custom_error! { #[derive(new)] #[allow(clippy::enum_variant_names)]
    pub DomainError
    PwdHashError {source: BcryptError} = "Failed to hash password",
    FieldValidationError {message: String} = "Failed to validate one or more fields",
    DbError {source: diesel::result::Error} = "Database error",
    DbPoolError {source: r2d2::Error} = "Failed to get connection from pool",
    PasswordError {cause: String} = "Failed to validate password - {cause}",
    EntityDoesNotExistError {message: String} = "Entity does not exist - {message}",
    BlockingError {source: actix_web::error::BlockingError} = "Thread pool error - {source}",
    AuthError {message: String} = "Authentication Error - {message}",
    JwtError {message: String} = "Jwt Error - {message}",
}

impl DomainError {
    pub fn anyhow_jwt(err: anyhow::Error) -> DomainError {
        DomainError::new_jwt_error(format!("{:#}", err))
    }
    pub fn anyhow_auth(message: &str, err: anyhow::Error) -> DomainError {
        DomainError::new_auth_error(format!("{}, {:#}", message, err))
    }
}

impl ResponseError for DomainError {
    fn error_response(&self) -> HttpResponse {
        let err = self;
        match self {
            DomainError::PwdHashError { source: _ } => {
                let _ = tracing::error!("{}", err);
                HttpResponse::InternalServerError()
                    .json(ApiResponse::failure(err.to_string()))
            }
            DomainError::DbError { source: _ } => {
                let _ = tracing::error!("{}", err);
                HttpResponse::InternalServerError()
                    .json(ApiResponse::failure("Error in database".to_owned()))
            }
            DomainError::DbPoolError { source: _ } => {
                let _ = tracing::error!("{}", err);
                HttpResponse::InternalServerError().json(ApiResponse::failure(
                    "Error getting database pool".to_owned(),
                ))
            }
            DomainError::PasswordError { cause: _ } => {
                let _ = tracing::error!("{}", err);
                HttpResponse::BadRequest()
                    .json(ApiResponse::failure(err.to_string()))
            }
            DomainError::EntityDoesNotExistError { message: _ } => {
                let _ = tracing::error!("{}", err);
                HttpResponse::NotFound()
                    .json(ApiResponse::failure(err.to_string()))
            }
            DomainError::BlockingError { source: _ } => {
                let _ = tracing::error!("{}", err);
                HttpResponse::InternalServerError()
                    .json(ApiResponse::failure("Blocking Error".to_owned()))
            }
            DomainError::AuthError { message: _ } => HttpResponse::Forbidden()
                .json(ApiResponse::failure(err.to_string())),
            DomainError::FieldValidationError { message: _ } => {
                let _ = tracing::error!("{}", err);
                HttpResponse::BadRequest()
                    .json(ApiResponse::failure(err.to_string()))
            }
            DomainError::JwtError { message: _ } => {
                let _ = tracing::error!("{}", err);
                HttpResponse::BadRequest()
                    .json(ApiResponse::failure(err.to_string()))
            }
        }
    }
}

// impl From<anyhow::Error> for DomainError {
//     fn from(err: anyhow::Error) -> DomainError {
//         //this should be safe to unwrap since our newtype
//         //does not allow negative values
//         format!("{:?}", err)
//     }
// }
