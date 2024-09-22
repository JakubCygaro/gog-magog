mod helpers;
mod objects;
pub mod resources;
use crate::{
    entity::{login_data, user_pfp},
    errors::{ServiceError, SessionValidationError},
};

use super::entity;
use super::entity::prelude::*;
use super::errors;
use super::session::TokenSession;
use actix_session::Session;
use actix_web::{self, http::header::ContentType, web::{self, Bytes}, HttpRequest, HttpResponse, Responder, ResponseError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use log::{debug, error, info, log, Level};
pub use objects::DbConnection;
use objects::{UserCreationData, UserDataResponse, UserLogin};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter};
use std::{borrow::BorrowMut, collections::HashMap, ops::Deref, str::FromStr, sync::Mutex};
use uuid::Uuid;
use validator::Validate;

static SESSION_ID: &str = "id";



#[actix_web::get("/")]
async fn hello_world() -> impl Responder {
    HttpResponse::ImATeapot()
}

#[actix_web::post("/logout")]
async fn user_logout(
    token_session: web::Data<Mutex<dyn TokenSession>>,
    session: Session,
) -> impl Responder {
    let Ok(Some(token)) = session.get::<Uuid>("id") else {
        return HttpResponse::BadRequest()
            .reason("no user session token cookie provided")
            .finish();
    };
    session.remove(SESSION_ID);

    let mut lock = token_session.lock();
    let token_session = lock.as_mut().unwrap();

    token_session.remove_user(&token);

    HttpResponse::Ok().reason("removed session").finish()
}

#[actix_web::post("/update")]
async fn user_update(
    db: web::Data<DbConnection>,
    update_data: web::Json<objects::UserUpdateData>,
    token_session: web::Data<Mutex<dyn TokenSession>>,
    session: Session,
) -> Result<HttpResponse, ServiceError> {
    let user = helpers::validate_session(&token_session, &session)?;
    let user_id = helpers::get_user_id(&user, &db).await?;

    let db = &db.db_connection;
    let Ok(Some(data_model)) = UserData::find_by_id(user_id).one(db).await else {
        error!("User data not found");
        return Ok(HttpResponse::InternalServerError().finish());
    };
    debug!("FOUND: {:?}", data_model);

    let mut model: entity::user_data::ActiveModel = data_model.into();

    update_data.0.update_model(&mut model);

    match model.update(db).await {
        Ok(_) => {
            info!("Updated");
            Ok(HttpResponse::Ok().reason("updated").finish())
        }
        Err(dberr) => {
            error!("{:?}", dberr);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[actix_web::post("/create")]
async fn user_create(
    creation_data: web::Json<UserCreationData>,
    app_data: web::Data<DbConnection>,
) -> impl Responder {
    log!(Level::Info, "user data: {:?}", creation_data.0);
    if let Err(e) = creation_data.validate() {
        error!("Validation errors\n {:?}", e.errors());
        let resp = objects::ValidationErrorResponse {
            reason: "Validation Failed".to_owned(),
            errors: e,
        };
        match serde_json::to_string_pretty(&resp) {
            Err(e) => {
                error!("Serialization error {}", e);
                return HttpResponse::InternalServerError().finish();
            }
            Ok(json) => {
                return HttpResponse::BadRequest()
                    .reason("validation error")
                    .body(json)
            }
        };
    }
    let creation = &creation_data.0;
    let db = &app_data.db_connection;

    use entity::login_data;
    let res = LoginData::find()
        .filter(login_data::Column::Login.eq(creation.login.as_str()))
        .one(db)
        .await;
    let response = match res {
        Ok(Some(_)) => {
            //log!(Level::Info, "user requested");
            HttpResponse::BadRequest()
                .reason("user already exists")
                .finish()
        }
        Ok(None) => {
            let salt = SaltString::generate(&mut OsRng);

            let argon2 = Argon2::default();

            let password_hash = argon2
                .hash_password(creation.password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            log!(
                Level::Debug,
                "login: {} , salt: {:?} , hash: {:?}",
                creation.login,
                salt,
                password_hash
            );

            let active = login_data::ActiveModel {
                login: sea_orm::ActiveValue::Set(creation.login.clone()),
                salt: sea_orm::ActiveValue::Set(salt.as_str().to_owned()),
                hash: sea_orm::ActiveValue::Set(password_hash),
                user_id: sea_orm::ActiveValue::Set(uuid::Uuid::new_v4().to_string()),
            };

            let data = entity::user_data::ActiveModel {
                user_id: active.user_id.clone(),
                description: sea_orm::ActiveValue::Set(Some("".to_owned())),
                ..Default::default()
            };
            if let Err(db_err) = LoginData::insert(active.clone()).exec(db).await {
                log!(
                    Level::Error,
                    "database insert error at create user for login: '{}' , err: '{:?}'",
                    creation.login,
                    db_err
                );
                return HttpResponse::InternalServerError().finish();
            };
            if let Err(db_err) = UserData::insert(data).exec(db).await {
                log!(
                    Level::Error,
                    "database insert error at create user data for login: '{}' , err: '{:?}'",
                    creation.login,
                    db_err
                );
                return match LoginData::delete(active).exec(db).await {
                    _ => HttpResponse::InternalServerError().finish(),
                };
            };
            HttpResponse::Created().finish()
        }
        Err(db_err) => {
            log!(
                Level::Error,
                "database find error at create user for login: '{}' , err: '{:?}'",
                creation.login,
                db_err
            );
            HttpResponse::InternalServerError().finish()
        }
    };
    response
}

#[actix_web::get("/exists")]
async fn user_exists(
    user_login: web::Query<UserLogin>,
    data: web::Data<DbConnection>,
) -> impl Responder {
    let db = &data.db_connection;
    use entity::login_data;
    match LoginData::find()
        .filter(login_data::Column::Login.eq(user_login.login.as_str()))
        .one(db)
        .await
    {
        Ok(Some(_)) => HttpResponse::Ok().reason("user exists").finish(),
        Ok(None) => HttpResponse::NotFound()
            .reason("user does not exist")
            .finish(),
        Err(err) => {
            log!(
                Level::Error,
                "user exists error for login '{}' err: {:?}",
                user_login.login,
                err
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::post("/token")]
async fn user_login_token(
    login_data: web::Json<UserCreationData>,
    data: web::Data<DbConnection>,
    token_session: web::Data<Mutex<dyn TokenSession>>,
    session: Session,
) -> Result<HttpResponse, errors::ServiceError> {
    use entity::login_data;
    use errors::ServiceError;
    let db = &data.db_connection;
    let login = &login_data.login;
    let user = LoginData::find()
        .filter(login_data::Column::Login.eq(login.as_str()))
        .one(db)
        .await;
    let Ok(Some(model)) = user else {
        return Err(ServiceError::UserNotFound);
    };

    let parsed_hash = PasswordHash::new(&model.hash).unwrap();
    if Argon2::default()
        .verify_password(login_data.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let mut lock = token_session.lock();
        let guard = lock.as_mut().unwrap();

        let token = guard.add_user(&model.login);
        session.insert(SESSION_ID, token.to_string()).unwrap();
        log!(Level::Debug, "token: {}", token.to_string());

        if let Some(t) = guard.get_user(&token) {
            log!(Level::Debug, "user session exitsts: {}", t);
        }

        Ok(HttpResponse::Accepted()
            .reason("password accepted")
            .finish())

    } else {
        session.remove(SESSION_ID);
        Err(ServiceError::WrongPassword)
    }
}

#[actix_web::get("/data")]
async fn user_data(
    data: web::Data<DbConnection>,
    token_session: web::Data<Mutex<dyn TokenSession>>,
    session: Session,
) -> Result<HttpResponse, ServiceError> {
    use entity::login_data;
    log!(Level::Debug, "user data");

    let usr_login = helpers::validate_session(&token_session, &session)?;

    let Ok(Some(usr)) = LoginData::find()
        .filter(login_data::Column::Login.eq(&usr_login))
        .one(&data.db_connection)
        .await
    else {
        log!(Level::Debug, "login data not found");
        return Ok(HttpResponse::NotFound().finish());
    };

    debug!("user_id: {}", &usr.user_id);

    let data = UserData::find()
        .filter(entity::user_data::Column::UserId.eq(&usr.user_id))
        .one(&data.db_connection)
        .await;

    match data {
        Ok(Some(data)) => {
            let data = UserDataResponse {
                login: usr.login,
                id: uuid::Uuid::from_str(&usr.user_id).unwrap(),
                description: data.description.unwrap_or_default(),
                gender: data.gender,
                created: data.created,
            };

            if let Ok(json) = serde_json::to_string(&data) {
                Ok(HttpResponse::Ok().body(json))
            } else {
                Ok(HttpResponse::InternalServerError().finish())
            }
        }
        Ok(None) => {
            debug!("user data not found despite login found");
            Ok(HttpResponse::NotFound().finish())
        }
        Err(e) => {
            error!("{:?}", e);
            Ok(HttpResponse::InternalServerError()
                .reason("an error occured")
                .finish())
        }
    }
}

#[actix_web::get("/get_pfp/{login}")]
async fn user_get_pfp(
    login: web::Path<String>,
    db: web::Data<DbConnection>,
) -> Result<HttpResponse, ServiceError> 
{
    let id = helpers::get_user_id(&login, &db).await?.to_string();
    match UserPfp::find_by_id(id).one(&db.db_connection).await? {
        Some(model) => {
            match model.data {
                Some(d) => {
                    debug!("send payload len: {}", d.len());
                    Ok(HttpResponse::Found().content_type(ContentType::jpeg()).body(d))
                },
                None => {
                    Ok(HttpResponse::NotFound().reason("user does not have a profile picture").finish())
                }
            }
        },
        None => {
            Ok(HttpResponse::NotFound().reason("user does not have a profile picture").finish())
        }
    }
}