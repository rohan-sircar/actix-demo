use actix_web::{HttpResponse, ResponseError};
use bcrypt::BcryptError;
use custom_error::custom_error;
// use derive_more::Display;
// use diesel::result::DatabaseErrorKind;
use crate::models::errors::*;
use r2d2;
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
    GenericError {cause: String} = "Generic Error - Reason: {cause}"
}

impl ResponseError for DomainError {
    fn error_response(&self) -> HttpResponse {
        let err = self;
        match self {
            DomainError::PwdHashError { source } => {
                HttpResponse::InternalServerError().json(ErrorModel {
                    error_code: 500,
                    reason: format!("{} {}", err.to_string(), source).as_str(),
                })
            }
            DomainError::DbError { source } => {
                HttpResponse::InternalServerError().json(ErrorModel {
                    error_code: 500,
                    reason: format!("{} {}", err.to_string(), source).as_str(),
                })
            }
            DomainError::DbPoolError { source } => {
                HttpResponse::InternalServerError().json(ErrorModel {
                    error_code: 500,
                    reason: format!("{} {}", err.to_string(), source).as_str(),
                })
            }
            DomainError::PasswordError { cause: _ } => {
                HttpResponse::BadRequest().json(ErrorModel {
                    error_code: 400,
                    reason: format!("{}", err.to_string()).as_str(),
                })
            }
            DomainError::GenericError { cause } => HttpResponse::BadRequest()
                .json(ErrorModel {
                    error_code: 400,
                    reason: format!("{} {}, ", err.to_string(), cause.clone())
                        .as_str(),
                }),
        }
    }
}
