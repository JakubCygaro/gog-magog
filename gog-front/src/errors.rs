use std::borrow::Borrow;
use std::{collections::HashMap, error::Error};

use thiserror::*;

#[derive(serde::Deserialize, Debug)]
pub struct ValidationErrorBody {
    pub reason: String,
    pub errors: HashMap<String, Vec<FieldError>>
}

#[derive(serde::Deserialize, Debug)]
pub struct FieldError {
    pub code: Option<String>,
    pub message: Option<String>,
    pub params: ValidationParams
}
#[derive(serde::Deserialize, Debug)]
pub struct ValidationParams {
    pub max: Option<i32>,
    pub min: Option<i32>
}

#[derive(Error, Debug)]
pub enum WebworksError {
    #[error("gloo_net error")]
    GlooError{
        #[from]
        err: gloo_net::Error
    },
    #[error("other error `{source:?}`")]
    Other{
        #[from]
        source: Box<dyn Error>
    },
    #[error("internal server error")]
    ServerError{
        status: String
    },
    #[error("unknown error `{msg:?}`")]
    Unknown{
        msg: String
    }
}

#[derive(Error, Debug)]
pub enum RegisterError{
    #[error("validation error")]
    ValidationError(ValidationErrorBody),
    #[error("webworks error")]
    Webworks{
        #[from]
        source: WebworksError
    }
}

#[derive(Error, Debug)]
pub enum UpdateUserError{
    #[error("validation error")]
    ValidationError(ValidationErrorBody),
    #[error("webworks error")]
    Webworks{
        #[from]
        source: WebworksError
    }
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("incorrect password supplied")]
    IncorrectPassword,
    #[error("no such user exists")]
    NoSuchUser,   
    #[error("webworks error")]
    Webworks{
        #[from]
        source: WebworksError
    }
}

#[derive(Error, Debug)]
pub enum PfpUploadError {
    #[error("webworks error")]
    Webworks {
        #[from]
        source: WebworksError
    },
    #[error("io error")]
    IoError {
        #[from]
        source: std::io::Error
    },
    #[error("file rejected `{reason}`")]
    Rejected{
        reason: String
    },
    #[error("web_sys error")]
    Websys {
        js_value: leptos::wasm_bindgen::JsValue
    }
}

#[derive(Error, Debug)]
pub enum CreatePostError {
    #[error("user is not logged in")]
    NotLoggedIn,
    #[error("validation error")]
    ValidationError(ValidationErrorBody),
    #[error("webworks error")]
    Webworks{
        #[from]
        source: WebworksError
    }
}
