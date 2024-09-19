use core::fmt;
use std::collections::HashMap;

use thiserror::*;

#[derive(serde::Deserialize, Debug)]
pub struct ValidationErrorBody {
    pub reason: String,
    pub errors: HashMap<String, Vec<FieldError>>
}

#[derive(serde::Deserialize, Debug)]
pub struct FieldError {
    pub code: String,
    pub message: String
}

#[derive(Error, Debug)]
pub enum RegisterError{
    #[error("internal server error")]
    ServerError{
        status: String
    },
    #[error("the user already exists")]
    ValidationError(ValidationErrorBody),
    #[error("unknown error `{msg:?}`")]
    Unknown{
        msg: String
    },
    #[error("gloo_net error")]
    GlooError{
        #[from]
        err: gloo_net::Error
    }
}


#[derive(Error, Debug)]
pub enum LoginError {
    #[error("internal server error")]
    ServerError{
        status: String
    },
    #[error("gloo_net error")]
    GlooError {
        #[from]
        err: gloo_net::Error
    },
    #[error("incorrect password supplied")]
    IncorrectPassword,
    #[error("no such user exists")]
    NoSuchUser,
    #[error("unknown error `{msg:?}`")]
    Unknown{
        msg: String
    }
}