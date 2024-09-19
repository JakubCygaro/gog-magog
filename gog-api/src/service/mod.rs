mod objects;
use crate::entity::login_data;

use super::entity;
use super::entity::prelude::*;
use super::errors;
use super::session::TokenSession;
use actix_session::Session;
use actix_web::{self, web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use log::{debug, error, info, log, Level};
pub use objects::DbConnection;
use objects::{UserCreationData, UserDataResponse, UserLogin};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use std::{borrow::BorrowMut, collections::HashMap, ops::Deref, str::FromStr, sync::Mutex};
use uuid::Uuid;
use validator::Validate;

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
    session.remove("id");

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
) -> impl Responder {
    let sess_result = session.get::<Uuid>("id");
    let Ok(Some(token)) = sess_result else {
        error!("{:?}", sess_result.err().unwrap());
        return HttpResponse::BadRequest()
            .reason("no session cookie provided")
            .finish();
    };

    let mut lock = token_session.lock();
    let token_session = lock.as_mut().unwrap();
    let Some(user) = token_session.get_user(&token) else {
        return HttpResponse::BadRequest()
            .reason("user not logged in")
            .finish();
    };
    let db = &db.db_connection;
    let Ok(Some(model)) = LoginData::find_by_id(user).one(db).await else {
        error!("User login not found");
        return HttpResponse::InternalServerError().finish();
    };
    let Ok(Some(data_model)) = UserData::find_by_id(model.user_id).one(db).await else {
        error!("User data not found");
        return HttpResponse::InternalServerError().finish();
    };
    debug!("FOUND: {:?}", data_model);

    let mut model: entity::user_data::ActiveModel = data_model.into();

    update_data.0.update_model(&mut model);

    match model.update(db).await {
        Ok(_) => {
            info!("Updated");
            HttpResponse::Ok().reason("updated").finish()
        }
        Err(dberr) => {
            error!("{:?}", dberr);
            HttpResponse::InternalServerError().finish()
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
                description: sea_orm::ActiveValue::Set("".to_owned()),
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
) -> Result<HttpResponse, errors::TokenError> {
    use entity::login_data;
    use errors::TokenError;
    let db = &data.db_connection;
    let login = &login_data.login;
    let user = LoginData::find()
        .filter(login_data::Column::Login.eq(login.as_str()))
        .one(db)
        .await;
    let Ok(Some(model)) = user else {
        return Err(TokenError::UserNotFound);
    };

    let parsed_hash = PasswordHash::new(&model.hash).unwrap();
    if Argon2::default()
        .verify_password(login_data.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let mut lock = token_session.lock();
        let guard = lock.as_mut().unwrap();

        let token = guard.add_user(&model.login);
        session.insert("id", token.to_string()).unwrap();
        log!(Level::Debug, "token: {}", token.to_string());

        if let Some(t) = guard.get_user(&token) {
            log!(Level::Debug, "user session exitsts: {}", t);
        }

        Ok(HttpResponse::Accepted()
            .reason("password accepted")
            .finish())
        // return match guard.add_user(&model.login) {
        //     Ok(token) => {

        //     }
        //     Err(e) => Err(TokenError::UserSessionError { source: e }),
        // };
    } else {
        session.remove("id");
        Err(TokenError::WrongPassword)
    }
}

#[actix_web::get("/data")]
async fn user_data(
    data: web::Data<DbConnection>,
    token_session: web::Data<Mutex<dyn TokenSession>>,
    session: Session,
) -> impl Responder {
    use entity::login_data;
    log!(Level::Debug, "user data");
    let Ok(Some(uuid_string)) = session.get::<String>("id") else {
        return HttpResponse::BadRequest()
            .reason("no user session token")
            .finish();
    };
    debug!("id: {}", uuid_string);
    let Ok(uuid) = uuid::Uuid::from_str(&uuid_string) else {
        return HttpResponse::InternalServerError()
            .reason("could not deserialize uuid")
            .finish();
    };
    debug!("token: {}", uuid);

    let mut lock = token_session.lock();
    let sess = lock.as_mut().unwrap();
    let db = &data.db_connection;
    let Some(usr_login) = sess.get_user(&uuid) else {
        debug!("no session for token: {}", uuid);
        return HttpResponse::Forbidden()
            .reason("no such user session")
            .finish();
    };
    // let Ok(a) = LoginData::find()
    //     .filter(login_data::Column::Login.eq(&usr_login))
    //     .find_with_related(UserData)
    //     .all(db)
    //     .await
    // else {
    //     return HttpResponse::InternalServerError().finish();
    // };
    // let Some((login_d, user_d)) = a.first() else {
    //     return HttpResponse::InternalServerError().finish();
    // };
    // let Some(user_d) = user_d.first() else {
    //     return HttpResponse::InternalServerError().finish();
    // };
    // let data = UserDataResponse {
    //     login: login_d.login.clone(),
    //     id: uuid::Uuid::from_str(&user_d.user_id).unwrap(),
    //     description: user_d.description.clone(),
    // };

    // return if let Ok(json) = serde_json::to_string(&data) {
    //     HttpResponse::Ok().body(json)
    // } else {
    //     HttpResponse::InternalServerError().finish()
    // };

    let Ok(Some(usr)) = LoginData::find()
        .filter(login_data::Column::Login.eq(&usr_login))
        .one(db)
        .await
    else {
        log!(Level::Debug, "login data not found");
        return HttpResponse::NotFound().finish();
    };

    debug!("user_id: {}", &usr.user_id);

    let data = UserData::find()
        .filter(entity::user_data::Column::UserId.eq(&usr.user_id))
        .one(db)
        .await;
    // else {
    //     log!(Level::Debug, "user data not found");
    //     return HttpResponse::NotFound().finish();
    // };

    match data {
        Ok(Some(data)) => {
            let data = UserDataResponse {
                login: usr.login,
                id: uuid::Uuid::from_str(&usr.user_id).unwrap(),
                description: data.description,
            };

            if let Ok(json) = serde_json::to_string(&data) {
                HttpResponse::Ok().body(json)
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Ok(None) => {
            debug!("user data not found despite login found");
            HttpResponse::NotFound().finish()
        }
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError()
                .reason("an error occured")
                .finish()
        }
    }
}
