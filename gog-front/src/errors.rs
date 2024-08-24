use thiserror::*;

#[derive(Error, Debug)]
pub enum RegisterError{
    #[error("internal server error")]
    ServerError{
        status: String
    },
    #[error("the user already exists")]
    UserAlreadyExists,
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