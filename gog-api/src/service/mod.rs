mod objects;
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
use log::{log, Level};
pub use objects::DbConnection;
use objects::{UserCreationData, UserDataResponse, UserLogin};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::{str::FromStr, sync::Mutex};

#[actix_web::get("/")]
async fn hello_world() -> impl Responder {
    "hello world"
}

#[actix_web::post("/create")]
async fn user_create(
    creation_data: web::Json<UserCreationData>,
    app_data: web::Data<DbConnection>,
) -> impl Responder {
    log!(Level::Info, "user data: {:?}", creation_data.0);
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
) -> Result<HttpResponse, errors::TokenError> {
    use entity::login_data;
    use errors::TokenError;
    let db = &data.db_connection;
    let login = &login_data.login;
    let user = LoginData::find()
        .filter(login_data::Column::Login.eq(login.as_str()))
        .one(db)
        .await;
    log!(Level::Debug, "token shit");
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
        return match guard.add_user(&model.login) {
            Ok(token) => {
                session.insert("id", token.to_string()).unwrap();
                log!(Level::Debug, "token: {}", token.to_string());

                if let Some(t) = guard.get_user(&token) {
                    log!(Level::Debug, "user session exitsts: {}", t);
                }

                Ok(HttpResponse::Accepted()
                    .reason("password accepted")
                    .finish())
            }
            Err(e) => Err(TokenError::UserSessionError { source: e }),
        };
    } else {
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
    log!(Level::Debug, "id: {}", uuid_string);
    let Ok(uuid) = uuid::Uuid::from_str(&uuid_string) else {
        return HttpResponse::InternalServerError()
            .reason("could not deserialize uuid")
            .finish();
    };
    log!(Level::Debug, "token: {}", uuid);

    let sess = &token_session.lock().unwrap();
    let db = &data.db_connection;
    let Some(usr_login) = sess.get_user(&uuid) else {
        log!(Level::Debug, "no session for token: {}", uuid);
        return HttpResponse::Forbidden()
            .reason("no such user session")
            .finish();
    };
    let Ok(Some(usr)) = LoginData::find()
        .filter(login_data::Column::Login.eq(&usr_login))
        .one(db)
        .await
    else {
        return HttpResponse::NotFound().finish();
    };
    let Ok(Some(data)) = UserData::find()
        .filter(entity::user_data::Column::UserId.eq(&usr.user_id))
        .one(db)
        .await
    else {
        return HttpResponse::NotFound().finish();
    };
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
