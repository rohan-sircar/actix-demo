use crate::models::misc::*;
use actix_web::{HttpResponse, ResponseError};
use bcrypt::BcryptError;
use custom_error::custom_error;
use std::convert::From;

custom_error! { #[derive(new)] #[allow(clippy::enum_variant_names)]
    pub DomainError
    PwdHashError {source: BcryptError} = "Failed to hash password",
    FieldValidationError {message: String} = "Failed to validate one or more fields",
    DbError {source: diesel::result::Error} = "Database error",
    DbPoolError {source: r2d2::Error} = "Failed to get connection from pool",
    BadInputError {message: String} = "Bad inputs to request: {message}",
    EntityDoesNotExistError {message: String} = "Entity does not exist - {message}",
    BlockingError {source: actix_web::error::BlockingError} = "Blocking error - {source}",
    AuthError {message: String} = "Authentication Error - {message}",
    JwtError {message: String} = "Jwt Error - {message}",
    RedisError {source: redis::RedisError} = "Redis Error = {source}",
    WsProtocolError {source: actix_ws::ProtocolError} = "WS Protocol Error = {source}",
    UninitializedError { message: String } = "A required component was not initialized - {message}",
    JoinError {source: tokio::task::JoinError } = "Join error - {source}",
    InternalError {message: String} = "An internal error occured - {message}"
}

impl DomainError {
    pub fn anyhow_jwt(err: anyhow::Error) -> DomainError {
        DomainError::new_jwt_error(format!("{err:#}"))
    }
    pub fn anyhow_auth(message: &str, err: anyhow::Error) -> DomainError {
        DomainError::new_auth_error(format!("{message}, {err:#}"))
    }
}

impl ResponseError for DomainError {
    fn error_response(&self) -> HttpResponse {
        let _ = tracing::error!("{:?}", self);
        match self {
            DomainError::PwdHashError { source: _ } => {
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new(self.to_string()))
            }
            DomainError::DbError { source: _ } => {
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Error in database".to_owned()))
            }
            DomainError::DbPoolError { source: _ } => {
                HttpResponse::InternalServerError().json(ErrorResponse::new(
                    "Error getting database pool".to_owned(),
                ))
            }
            DomainError::BadInputError { message: _ } => {
                HttpResponse::BadRequest()
                    .json(ErrorResponse::new(self.to_string()))
            }
            DomainError::EntityDoesNotExistError { message: _ } => {
                HttpResponse::NotFound()
                    .json(ErrorResponse::new(self.to_string()))
            }
            DomainError::BlockingError { source: _ } => {
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Blocking Error".to_owned()))
            }
            DomainError::AuthError { message: _ } => HttpResponse::Forbidden()
                .json(ErrorResponse::new(self.to_string())),
            DomainError::FieldValidationError { message: _ } => {
                HttpResponse::BadRequest()
                    .json(ErrorResponse::new(self.to_string()))
            }
            DomainError::JwtError { message: _ } => HttpResponse::BadRequest()
                .json(ErrorResponse::new(self.to_string())),
            DomainError::RedisError { source: _ } => {
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Failure in Redis"))
            }
            DomainError::UninitializedError { message } => {
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new(message))
            }
            DomainError::WsProtocolError { source: _ } => {
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Websocket Protocol Failure"))
            }
            DomainError::InternalError { message } => {
                HttpResponse::InternalServerError().json(ErrorResponse::new(
                    format!("An internal error occured {message}"),
                ))
            }
            DomainError::JoinError { source: _ } => {
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Join Error"))
            }
        }
    }
}
