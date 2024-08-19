use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenError {
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
        source: SessionError,
    },
}

impl ResponseError for TokenError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            &TokenError::WrongPassword => {
                HttpResponse::Forbidden().reason("wrong password").finish()
            }
            &TokenError::UserNotFound => {
                HttpResponse::BadRequest().reason("user not found").finish()
            }
            &TokenError::SessionError { source: _ } => HttpResponse::InternalServerError()
                .reason("session error")
                .finish(),
            &TokenError::UserSessionError { source: _ } => HttpResponse::InternalServerError()
                .reason("user session error")
                .finish(),
        }
    }
}

#[derive(Error, Debug)]
pub enum SessionError {}
