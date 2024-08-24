use anyhow::Error;
use anyhow::{Result, anyhow};
use leptos;
use gloo_net::http::Request;
use super::data::*;
use super::errors::*;
const URL_BASE: &'static str = "http://localhost:8081/";

pub async fn get_token(data: &LoginData) -> Result<(), LoginError> {

    let body = serde_json::to_string(data)
        .or_else(|e| Err(LoginError::Unknown { msg: e.to_string() }))?;


    let resp = Request::post(&(URL_BASE.to_owned() + "user/token"))
        .header("Content-Type", "application/json")
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .body(body)
        .or_else(|e| Err(LoginError::GlooError { err: e }))?
        .send()
        .await;
    match resp {
        Ok(response) => {
            match response.status() {
                202 => Ok(()),
                500 => Err(LoginError::ServerError { status: response.status_text() }),
                403 => Err(LoginError::IncorrectPassword),
                400 => Err(LoginError::NoSuchUser),
                _ => Err(LoginError::Unknown { msg: response.status_text() })
            }
        },
        Err(e) => Err(LoginError::GlooError { err: e })
    }
}

pub async fn register(user_creation: &UserCreationData) -> Result<(), RegisterError> {
    let body = serde_json::to_string(&user_creation).unwrap();
    let resp = Request::post(&(URL_BASE.to_owned()+ "user/create"))
        .header("Content-Type", "application/json")
        .body(&body)
        .or_else(|e| Err(RegisterError::GlooError { err: e }))?
        .send()
        .await;
    match resp {
        Ok(resp) => {
            match resp.status() {
                400 => Err(RegisterError::UserAlreadyExists),
                201 => Ok(()),
                _ => Err(RegisterError::ServerError { status: resp.status_text() }),
            }
        },
        Err(e) => Err(RegisterError::Unknown { msg: e.to_string() }),
    }
        
}

pub async fn get_user_data() -> Option<UserData> {
    let response = Request::get("http://localhost:8081/user/data")
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .send()
        .await
        .or_else(|e| Err(anyhow!(e)));

    let Ok(response) = response else {
        return None;
    };

    let data = response.json::<UserData>().await;

    match data {
        Ok(d) => Some(d),
        Err(_) => None
    }
        
}