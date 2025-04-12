use uuid::Uuid;
use tokio::sync::mpsc;
use anyhow::{Result, anyhow};
use gloo_net::http::Request;
use web_sys::wasm_bindgen::JsCast;
use super::data::*;
use super::errors::*;
use leptos::{web_sys, wasm_bindgen};


pub type WebworksResult<T> = Result<T, WebworksError>;
const URL_BASE: &str = "http://localhost:8081/";

#[derive(serde::Serialize, Debug, Clone)]
pub struct PostsFilter {
    pub username: Option<String>,
    pub limit: Option<u64>
}
pub async fn get_token(data: &LoginData) -> Result<(), LoginError> {

    let body = serde_json::to_string(data).map_err(|e| WebworksError::Other { source: Box::new(e) })?;


    let resp = Request::post(&(URL_BASE.to_owned() + "user/token"))
        .header("Content-Type", "application/json")
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .body(body).map_err(|e| WebworksError::GlooError { err: e })?
        .send()
        .await;
    match resp {
        Ok(response) => {
            match response.status() {
                202 => Ok(()),
                500 => Err(WebworksError::ServerError { status: response.status_text() })?,
                403 => Err(LoginError::IncorrectPassword),
                400 => Err(LoginError::NoSuchUser),
                _ => Err(WebworksError::Unknown { msg: response.status_text() })?
            }
        },
        Err(e) => Err(WebworksError::GlooError { err: e })?
    }
}

pub async fn register(user_creation: &UserCreationData) -> Result<(), RegisterError> {
    let body = serde_json::to_string(&user_creation).unwrap();
    let resp = Request::post(&(URL_BASE.to_owned()+ "user/create"))
        .header("Content-Type", "application/json")
        .body(&body).map_err(|e| WebworksError::Other { source: Box::new(e) })?
        .send()
        .await;
    match resp {
        Ok(resp) => {
            match resp.status() {
                400 => {
                    let Ok(body) = resp.json::<ValidationErrorBody>().await else {
                        return Err(WebworksError::Unknown { msg: "failed to read json response".to_string() })?
                    };
                    Err(RegisterError::ValidationError(body))
                },
                201 => Ok(()),
                _ => Err(WebworksError::ServerError { status: resp.status_text() })?,
            }
        },
        Err(e) => Err(WebworksError::Other { source: Box::new(e) })?,
    }

}

pub async fn get_user_data() -> Option<UserData> {
    let response = Request::get(&(URL_BASE.to_owned()+ "user/data"))
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .send()
        .await.map_err(|e| anyhow!(e));

    let Ok(response) = response else {
        return None;
    };

    let data = response.json::<UserData>().await;

    match data {
        Ok(d) => Some(d),
        Err(_) => None
    }

}

pub async fn update_user_data(data: &UserData) -> Result<(), UpdateUserError> {
    // let body = serde_json::to_string(&data);
    let response = Request::post(&(URL_BASE.to_owned()+ "user/update"))
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .json(data).map_err(|e| WebworksError::Other { source: Box::new(e) })?
        .send()
        .await.map_err(|e| WebworksError::Other { source: Box::new(e) })?;
    // ok ok
    if response.status() == 200 {
        Ok(())
    } else if response.status() == 400 {
        let Ok(body) = response.json::<ValidationErrorBody>().await else {
            return Err(WebworksError::Unknown { msg: "failed to read json response".to_string() })?
        };
        Err(UpdateUserError::ValidationError(body))
    } else {
        Err(WebworksError::Unknown{ msg: "Update data error".to_string() })?
    }
}

pub async fn logout_user() -> Result<()> {
    let _resp = Request::post(&(URL_BASE.to_owned()+ "user/logout"))
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .send()
        .await.map_err(|e| anyhow!(e))?;
    Ok(())
}

pub fn get_pfp_url_for_login(login: &str) -> String {
    let r= format!("{}user/get_pfp/{}#{}", URL_BASE, login, chrono::Utc::now().timestamp());
    r
    // format!("{}user/get_pfp/{}", URL_BASE, login)
}

pub async fn upload_new_pfp(file: web_sys::File) -> mpsc::Receiver<Result<(), PfpUploadError>> {
    use wasm_bindgen::prelude::*;

    use web_sys::{Request, RequestInit, Response};

    let (sender, reciever) = mpsc::channel::<Result<(), PfpUploadError>>(1);
    let file_name = file.name();
    if !file_name.ends_with(".jpg") && !file_name.ends_with(".jpeg") {
        sender.send(Err(PfpUploadError::Rejected { reason: "the file is not a valid jpg/jpeg".to_string() })).await
            .unwrap();
        return reciever;
    }
    if file.size() > 25_000.0 {
        sender.send(Err(PfpUploadError::Rejected { reason: "this file is to big and will not be sent".to_string() })).await
            .unwrap();
        return reciever;
    }

    let reader = web_sys::FileReader::new().map_err(|e| PfpUploadError::Websys { js_value: e }).unwrap();

    reader.read_as_array_buffer(&file).unwrap();
    let sender_clone = sender.clone();
    let sender_clone2 = sender_clone.clone();
    let sender_clone3 = sender_clone.clone();
    let reader_clone = reader.clone();

    // let resp_value = JsFuture::from();
    let resolve = js_closure!(move |v: JsValue|{
        let response = Response::from(v);
        match response.status() {
            200 => sender_clone2.blocking_send(Ok(())).unwrap(),
            400 => sender_clone2.blocking_send(Err(PfpUploadError::Rejected{ reason: response.status_text()})).unwrap(),
            _ => sender_clone2.blocking_send(Err(PfpUploadError::Webworks { source: WebworksError::Unknown { msg: "unknown error".to_owned() } })).unwrap()
        };
    });
    let failure = js_closure!(move |v: JsValue|{
        sender_clone3.blocking_send(Err(PfpUploadError::Websys { js_value: v })).unwrap();
    });

    let load= js_closure!(move |e: web_sys::Event| {

        let res = reader_clone.result().unwrap();
        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_credentials(web_sys::RequestCredentials::Include);
        opts.set_body(&res);
        let url = format!("{}user/upload_pfp", URL_BASE);
        let request = Request::new_with_str_and_init(&url, &opts).unwrap();
        request.headers()
            .set("content-type", "image/jpg").unwrap();

        let window = web_sys::window().unwrap();

        let _ = window.fetch_with_request(&request)
            .then2(&resolve, &failure);

    });

    let error = js_closure!( move |e: web_sys::Event| {
        sender.blocking_send(Err(PfpUploadError::from(WebworksError::Unknown { msg: "error reading file".to_owned() }))).unwrap();
    });

    reader.add_event_listener_with_callback("load", load.as_ref().unchecked_ref()).unwrap();
    reader.add_event_listener_with_callback("error", error.as_ref().unchecked_ref()).unwrap();

    load.forget(); error.forget();

    reciever
}

pub async fn load_posts(amount: i32, filter: Option<&PostsFilter>) -> Result<Vec<PostData>, WebworksError> {
    let text;
    if let Some(filter) = filter {
        let resp = Request::post(&format!("{}posts/filter", URL_BASE));
        let resp = resp.json(filter).unwrap();
        let resp = resp.send().await?;
        text = resp.text().await?;
    } else {
        let resp = Request::get(&format!("{}posts/newest/{}", URL_BASE, amount));
        let resp = resp.send().await?;
        text = resp.text().await?;
    }
    let json = serde_json::from_str::<Vec<PostData>>(&text).unwrap();
    Ok(json)
}
pub async fn get_post(pid: uuid::Uuid) -> Result<PostData, WebworksError> {
    let resp = Request::get(&format!("{}posts/id/{}", URL_BASE, pid));
    let resp = resp.send().await?;
    let text = resp.text().await?;
    leptos::logging::debug_warn!("{}", text);
    let json = serde_json::from_str::<PostData>(&text).unwrap();
    Ok(json)
}

pub async fn create_post(data: PostCreationData) -> Result<(), CreatePostError> {
    let resp = Request::post(&format!("{}posts/create", URL_BASE))
        .credentials(leptos::web_sys::RequestCredentials::Include)
        .json(&data).map_err(|e| WebworksError::Other { source: Box::new(e) })?
        .send()
        .await.map_err(|e| WebworksError::Other { source: Box::new(e) })?;
    match resp.status() {
        201 => Ok(()),
        400 => Err(CreatePostError::ValidationError(resp.json::<ValidationErrorBody>().await.map_err(|e| WebworksError::Other { source: Box::new(e) })?)),
        _ => Err(CreatePostError::NotLoggedIn)
    }
}

pub async fn get_user_profile(query: super::data::UserProfileQuery) -> WebworksResult<UserData> {

    let req_str = format!("{}{}{}{}", URL_BASE, "user/profile?",
        query.name.map_or("".to_owned(), |n| {
            format!("username={}", n)
        }),
        query.id.map_or("".to_owned(), |id| {
            format!("user_id={}", id)
        })
    );

    let response = Request::get(&req_str)
        .send()
        .await.map_err(|e| WebworksError::Other { source: Box::new(e) })?;
    let data = response.json::<UserData>().await
        .map_err(|e| WebworksError::Other { source: Box::new(e) })?;

    Ok(data)
}

pub async fn load_comments(pid: Uuid, limit: i32) -> Result<Vec<CommentData>, WebworksError> {
    let query_str = format!("{}{}pid={}&limit={}", URL_BASE, "comments?", pid.to_string(), limit);
    leptos::logging::log!("query_str: {}", query_str);
    let response = Request::get(&query_str)
        .send()
        .await?;
    let text = response.text().await?;
    let json = serde_json::from_str::<Vec<CommentData>>(&text).expect("expected comment data list response json from api");
    Ok(json)
}
pub async fn leave_comment(ccdata: CommentCreationData) -> Result<(), WebworksError> {
    let request_str = format!("{}{}", URL_BASE, "comments/post");
    Request::post(&request_str)
        .json(&ccdata)?
        .send()
        .await?;
    Ok(())
}
