use actix_web::{HttpResponse, ResponseError};
use bcrypt::BcryptError;
use custom_error::custom_error;
// use derive_more::Display;
// use diesel::result::DatabaseErrorKind;
use crate::models::errors::*;
use r2d2;
use std::convert::From;
// use std::error::Error;

// pub enum DomainError {
//     #[display(fmt = "PasswordHashError")]
//     PwdHashError,
//     #[display(fmt = "Bad Id")]
//     IdError,
//     #[display(fmt = "Generic Error")]
//     GenericError,
//     DuplicateValue,
// }

// impl Error for DomainError {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         // Generic error, underlying cause isn't tracked.
//         None
//     }
// }

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

custom_error! { pub DomainError
    PwdHashError {source: BcryptError} = "Failed to has password",
    DbError {source: diesel::result::Error} = "Database error",
    DbPoolError {source: r2d2::Error} = "Failed to get connection from pool",
    GenericError {cause: String} = "Generic Error - Reason: {cause}"
}

impl ResponseError for DomainError {
    fn error_response(&self) -> HttpResponse {
        match self {
            DomainError::PwdHashError { source } => {
                HttpResponse::InternalServerError().json(ErrorModel {
                    status_code: 500,
                    reason: format!(
                        "{} {}",
                        "Unexpected Error - Failed to hash password", source
                    ),
                })
            }
            DomainError::DbError { source } => {
                HttpResponse::InternalServerError().json(ErrorModel {
                    status_code: 500,
                    reason: format!("{} {}", "Unexpected Database Error", source),
                })
            }
            DomainError::DbPoolError { source } => {
                HttpResponse::InternalServerError().json(ErrorModel {
                    status_code: 500,
                    reason: format!(
                        "{} {}",
                        "Unexpected Error - Failed to get connection from pool", source
                    ),
                })
            }
            DomainError::GenericError { cause } => HttpResponse::BadRequest().json(ErrorModel {
                status_code: 400,
                reason: format!(
                    "{} {}, ",
                    "Unexpected Database Error - ".to_owned(),
                    cause.clone()
                ),
            }),
        }
    }
}
