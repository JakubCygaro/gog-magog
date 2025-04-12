pub mod comments;
mod helpers;
mod objects;
pub mod posts;
pub mod resources;
use super::entity;
use super::entity::prelude::*;
use super::errors;
use super::session::TokenSession;
use crate::{cache::ResourceCache, entity::login_data, errors::ServiceError};
use actix_session::Session;
use actix_web::{
    self,
    http::header::{self, CacheDirective, ContentType},
    web::{self},
    HttpResponse, Responder,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use gog_commons as commons;
use gog_commons::data_structures::UserCreationData;
use gog_commons::data_structures::UserDataResponse;
use gog_commons::data_structures::UserLogin;
use log::{debug, error, info, log, Level};
pub use objects::DbConnection;
use objects::UserProfileQuery;
use objects::UserUpdateDataExt;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, QueryFilter, TransactionTrait,
};
use serde::Deserialize;
use std::{error::Error, str::FromStr, sync::Mutex};
use uuid::Uuid;
use validator::Validate;
type ServiceResult = Result<HttpResponse, ServiceError>;

static SESSION_ID: &str = "id";
pub fn configure_service(cfg: &mut web::ServiceConfig) {
    use actix_web::guard;
    cfg.service(hello_world);
    let user_scope = web::scope("/user")
        .service(user_create)
        .service(user_exists)
        .service(user_login_token)
        .service(user_data)
        .service(user_update)
        .service(user_logout)
        .service(
            web::resource("/upload_pfp")
                .guard(guard::Header("content-type", "image/jpg"))
                .guard(guard::Post())
                .route(web::post().to(resources::user_upload_pfp)),
        )
        .service(user_get_pfp)
        // .service(user_profile_name)
        // .service(user_profile_id)
        .service(user_profile);
    cfg.service(user_scope);
}

#[actix_web::get("profile")]
async fn user_profile(
    query: web::Query<UserProfileQuery>,
    db: web::Data<DbConnection>,
) -> Result<HttpResponse, ServiceError> {
    let query = query.into_inner();

    let user = match (query.user_id, query.username) {
        (Some(id), None) => {
            LoginData::find()
                .filter(login_data::Column::UserId.eq(id))
                .one(&db.db_connection)
                .await?
        }
        (_, Some(login)) => LoginData::find_by_id(login).one(&db.db_connection).await?,
        _ => None,
    };

    let Some(login_data) = user else {
        return Err(ServiceError::UserNotFound);
    };

    let data = UserData::find_by_id(login_data.user_id)
        .one(&db.db_connection)
        .await?;

    match data {
        Some(model) => {
            let resp = UserDataResponse {
                created: model.created,
                description: model.description.unwrap_or_default(),
                login: login_data.login,
                gender: model.gender,
                id: model.user_id,
            };
            let json = serde_json::to_string(&resp).or_else(|e| {
                error!("user_profile_id serialization error {:?}", e);
                Err(ServiceError::ServerError {
                    source: Box::new(e),
                })
            })?;
            Ok(HttpResponse::Found()
                .append_header(header::ContentType::json())
                .body(json))
        }
        None => Err(ServiceError::UserNotFound),
    }
}

// #[actix_web::get("/profile/name/{login}")]
// async fn user_profile_name(
//     login: web::Path<String>,
//     db: web::Data<DbConnection>,
// ) -> Result<HttpResponse, ServiceError> {
//     let login = login.into_inner();
//     let usr = LoginData::find_by_id(&login).one(&db.db_connection).await?;

//     let Some(usr) = usr else {
//         return Ok(HttpResponse::NotFound()
//             .reason("user does not exist")
//             .finish());
//     };

//     let data = UserData::find_by_id(usr.user_id)
//         .one(&db.db_connection)
//         .await?
//         .unwrap();

//     let resp = UserDataResponse {
//         created: data.created,
//         description: data.description.unwrap_or_default(),
//         login: login,
//         gender: data.gender,
//         id: data.user_id,
//     };

//     let json = serde_json::to_string(&resp).or_else(|e| {
//         error!("user_profile_name serialization error: {:?}", e);
//         Err(ServiceError::ServerError {
//             source: Box::new(e),
//         })
//     })?;

//     Ok(HttpResponse::Found()
//         .append_header(header::ContentType::json())
//         .body(json))
// }

// #[actix_web::get("/profile/id/{id}")]
// async fn user_profile_id(id: web::Path<Uuid>, db: web::Data<DbConnection>) -> ServiceResult {
//     // let id = Uuid::from_str(&id)
//     //     .or_else(|e| {
//     //         Err(ServiceError::UserNotFound)
//     //     })?;
//     let id = id.into_inner();
//     let Some(user) = LoginData::find()
//         .filter(login_data::Column::UserId.eq(id))
//         .one(&db.db_connection)
//         .await?
//     else {
//         return Err(ServiceError::UserNotFound);
//     };

//     let data = UserData::find_by_id(id).one(&db.db_connection).await?;

//     match data {
//         Some(model) => {
//             let resp = UserDataResponse {
//                 created: model.created,
//                 description: model.description.unwrap_or_default(),
//                 login: user.login,
//                 gender: model.gender,
//                 id: model.user_id,
//             };
//             let json = serde_json::to_string(&resp).or_else(|e| {
//                 error!("user_profile_id serialization error {:?}", e);
//                 Err(ServiceError::ServerError {
//                     source: Box::new(e),
//                 })
//             })?;
//             Ok(HttpResponse::Found()
//                 .append_header(header::ContentType::json())
//                 .body(json))
//         }
//         None => Err(ServiceError::UserNotFound),
//     }
// }

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
    update_data: web::Json<gog_commons::data_structures::UserUpdateData>,
    token_session: web::Data<Mutex<dyn TokenSession>>,
    session: Session,
) -> Result<HttpResponse, ServiceError> {
    let update_data = update_data.into_inner();
    if let Err(e) = update_data.validate() {
        let resp = commons::data_structures::ValidationErrorResponse {
            reason: "Validation Failed".to_owned(),
            errors: e,
        };
        let json = serde_json::to_string(&resp).unwrap();
        debug!("{}", json);
        return Ok(HttpResponse::BadRequest().body(json));
    }

    let user = helpers::validate_session(&token_session, &session)?;
    let user_id = helpers::get_user_id(&user, &db).await?;

    let db = &db.db_connection;
    let Ok(Some(data_model)) = UserData::find_by_id(user_id).one(db).await else {
        error!("User data not found");
        return Ok(HttpResponse::InternalServerError().finish());
    };
    debug!("FOUND: {:?}", data_model);

    let mut model: entity::user_data::ActiveModel = data_model.into();

    update_data.update_model(&mut model);

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
        let resp = commons::data_structures::ValidationErrorResponse {
            reason: "Validation Failed".to_owned(),
            errors: e,
        };
        return HttpResponse::BadRequest().json(resp);
        // match serde_json::to_string_pretty(&resp) {
        //     Err(e) => {
        //         error!("Serialization error {}", e);
        //         return HttpResponse::InternalServerError().finish();
        //     }
        //     Ok(json) => {
        //         return HttpResponse::BadRequest()
        //             .reason("validation error")
        //             .body(json)
        //     }
        // };
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
                user_id: sea_orm::ActiveValue::Set(uuid::Uuid::new_v4()),
            };

            let data = entity::user_data::ActiveModel {
                user_id: active.user_id.clone(),
                description: sea_orm::ActiveValue::Set(Some("".to_owned())),
                created: ActiveValue::Set(Some(chrono::Utc::now())),
                ..Default::default()
            };

            let res = db
                .transaction::<_, (), DbErr>(|txn| {
                    Box::pin(async move {
                        active.insert(txn).await?;
                        data.insert(txn).await?;
                        Ok(())
                    })
                })
                .await;

            match res {
                Ok(_) => HttpResponse::Created().finish(),
                Err(t_err) => {
                    error!("Could not create new user: {}", t_err.to_string());
                    HttpResponse::InternalServerError()
                        .reason("could not create user")
                        .finish()
                }
            }

            // if let Err(db_err) = LoginData::insert(active.clone()).exec(db).await {
            //     log!(
            //         Level::Error,
            //         "database insert error at create user for login: '{}' , err: '{:?}'",
            //         creation.login,
            //         db_err
            //     );
            //     return HttpResponse::InternalServerError().finish();
            // };
            // if let Err(db_err) = UserData::insert(data).exec(db).await {
            //     log!(
            //         Level::Error,
            //         "database insert error at create user data for login: '{}' , err: '{:?}'",
            //         creation.login,
            //         db_err
            //     );
            //     return match LoginData::delete(active).exec(db).await {
            //         _ => HttpResponse::InternalServerError().finish(),
            //     };
            // };
            // HttpResponse::Created().finish()
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
    session.remove(SESSION_ID);

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
        .filter(entity::user_data::Column::UserId.eq(usr.user_id))
        .one(&data.db_connection)
        .await;

    match data {
        Ok(Some(data)) => {
            let data = UserDataResponse {
                login: usr.login,
                id: usr.user_id,
                description: data.description.unwrap_or_default(),
                gender: data.gender,
                created: data.created,
            };

            if let Ok(json) = serde_json::to_string(&data) {
                Ok(HttpResponse::Ok()
                    .append_header(header::ContentType::json())
                    .body(json))
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

static DEFAULT_PFP_PATH: &str = "data/default_pfp.jpeg";

#[actix_web::get("/get_pfp/{login}")]
async fn user_get_pfp(
    login: web::Path<String>,
    db: web::Data<DbConnection>,
    cache: web::Data<Mutex<ResourceCache>>,
) -> Result<HttpResponse, ServiceError> {
    let id = if let Ok(uuid) = uuid::Uuid::from_str(&login) {
        uuid
    } else {
        helpers::get_user_id(&login, &db).await?
    };
    match UserPfp::find_by_id(id).one(&db.db_connection).await? {
        Some(entity::user_pfp::Model {
            user_id: _,
            data: Some(d),
        }) => Ok(HttpResponse::Found()
            .content_type(ContentType::jpeg())
            .append_header(header::CacheControl(vec![
                CacheDirective::MaxAge(0),
                CacheDirective::MustRevalidate,
                CacheDirective::NoStore,
            ]))
            .body(d)),
        _ => {
            let mut lock = cache.lock();
            let cache = lock.as_mut().unwrap();
            let pfp = cache.get_or_load(DEFAULT_PFP_PATH).await?;
            debug!("pfp: {}", pfp.len());
            Ok(HttpResponse::Found()
                .content_type(ContentType::jpeg())
                .append_header(header::CacheControl(vec![
                    CacheDirective::MaxAge(0),
                    CacheDirective::MustRevalidate,
                    CacheDirective::NoStore,
                ]))
                .body(pfp))
        }
    }
}
