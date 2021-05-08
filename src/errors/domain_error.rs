use actix_web::{HttpResponse, ResponseError};
use bcrypt::BcryptError;
use custom_error::custom_error;
// use derive_more::Display;
// use diesel::result::DatabaseErrorKind;
use crate::models::errors::*;
use std::convert::From;

// impl From<DBError> for DomainError {
//     fn from(error: DBError) -> DomainError {
//         // We only care about UniqueViolations
//         match error {
//             DBError::DatabaseError(kind, info) => {
//                 let message = info.details().unwrap_or_else(|| info.message()).to_string();
//                 match kind {
//                     DatabaseErrorKind::UniqueViolation => DomainError::DuplicateValue(message),
//                     _ => DomainError::GenericError(message),
//                 }
//             }
//             _ => DomainError::GenericError(String::from("Some database error occured")),
//         }
//     }
// }

custom_error! { #[derive(new)] pub DomainError
    PwdHashError {source: BcryptError} = "Failed to hash password",
    DbError {source: diesel::result::Error} = "Database error",
    DbPoolError {source: r2d2::Error} = "Failed to get connection from pool",
    PasswordError {cause: String} = "Failed to validate password - {cause}",
    EntityDoesNotExistError {message: String} = "Entity does not exist - {message}",
    ThreadPoolError {message: String} = "Thread pool error - {message}",
    AuthError {message: String} = "Authentication Error - {message}"
}

impl ResponseError for DomainError {
    fn error_response(&self) -> HttpResponse {
        let err = self;
        match self {
            DomainError::PwdHashError { source: _ } => {
                HttpResponse::InternalServerError().json(ErrorModel {
                    // error_code: 500,
                    success: false,
                    reason: err.to_string(),
                })
            }
            DomainError::DbError { source: _ } => {
                log::error!("{}", err);
                HttpResponse::InternalServerError().json(ErrorModel {
                    // error_code: 500,
                    success: false,
                    reason: "Error in database".to_owned(),
                })
            }
            DomainError::DbPoolError { source: _ } => {
                log::error!("{}", err);
                HttpResponse::InternalServerError().json(ErrorModel {
                    // error_code: 500,
                    success: false,
                    reason: "Error getting database pool".to_owned(),
                })
            }
            DomainError::PasswordError { cause: _ } => {
                HttpResponse::BadRequest().json(ErrorModel {
                    // error_code: 400,
                    success: false,
                    reason: err.to_string(),
                })
            }
            DomainError::EntityDoesNotExistError { message: _ } => {
                HttpResponse::Accepted().json(ErrorModel {
                    // error_code: 400,
                    success: false,
                    reason: err.to_string(),
                })
            }
            DomainError::ThreadPoolError { message: _ } => {
                log::error!("{}", err);
                HttpResponse::InternalServerError().json(ErrorModel {
                    // error_code: 400,
                    success: false,
                    reason: "Thread pool error occurred".to_owned(),
                })
            }
            DomainError::AuthError { message: _ } => {
                HttpResponse::Accepted().json(ErrorModel {
                    // error_code: 400,
                    success: false,
                    reason: err.to_string(),
                })
            }
        }
    }
}
