use std::error::Error;

use actix_web::{HttpResponse, ResponseError};
use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("User not found")]
    UserNotFound,

    #[error("Wrong password")]
    WrongPassword,

    #[error("Session error")]
    SessionError {
        #[from]
        source: actix_session::SessionInsertError,
    },
    #[error("User session error")]
    UserSessionError {
        #[from]
        source: SessionValidationError,
    },
    #[error("UserId error")]
    UserIdError {
        #[from]
        source: UserIdError,
    },
    #[error("Database error")] 
    DatabaseError {
        #[from]
        source: DbErr
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            ServiceError::WrongPassword => {
                HttpResponse::Forbidden().reason("wrong password").finish()
            }
            ServiceError::UserNotFound => {
                HttpResponse::BadRequest().reason("user not found").finish()
            }
            ServiceError::SessionError { source: _ } => HttpResponse::InternalServerError()
                .reason("session error")
                .finish(),
            ServiceError::UserSessionError { source: s } => s.error_response(),
            ServiceError::UserIdError { source: s } => s.error_response(),
            Self::DatabaseError { source } => 
                HttpResponse::InternalServerError().reason("database error").finish()
        }
    }
}

#[derive(Error, Debug)]
pub enum SessionValidationError {
    #[error("No user session")]
    NoSession,
    #[error("No user session cookie")]
    NoCookie,
    #[error("Other error")]
    Other {
        #[from]
        source: Box<dyn Error>,
    },
}

impl ResponseError for SessionValidationError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::NoCookie => HttpResponse::BadRequest()
                .reason("user session cookie not provided")
                .finish(),
            Self::NoSession => HttpResponse::Forbidden()
                .reason("no active user session")
                .finish(),
            Self::Other { source: _ } => HttpResponse::InternalServerError()
                .reason("an internal error has occured")
                .finish(),
        }
    }
}

#[derive(Error, Debug)]
pub enum UserIdError {
    #[error("No user in database")]
    NoUser,
    #[error("Database error")]
    DatabaseError {
        #[from]
        source: sea_orm::DbErr,
    },
}

impl ResponseError for UserIdError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::NoUser => HttpResponse::BadRequest()
                .reason("user does not exist")
                .finish(),
            Self::DatabaseError { source: _ } => HttpResponse::InternalServerError()
                .reason("database error")
                .finish(),
        }
    }
}
