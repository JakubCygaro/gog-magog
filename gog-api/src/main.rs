mod args;
mod entity;
mod errors;
mod migrator;
mod session;
use actix_cors::Cors;
use args as run_args;
use clap::Parser;
use core::sync;
use std::{borrow::BorrowMut, io::Read, str::FromStr, sync::Mutex};

use actix_session::{
    storage::{CookieSessionStore, SessionStore},
    Session, SessionMiddleware,
};
use actix_web::{
    self,
    cookie::Key,
    http::{header::ContentType, StatusCode},
    middleware::Logger,
    web::{self, post, service},
    App, HttpResponse, HttpServer, Responder,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use entity::{login_data, prelude::*};
use log::{log, Level};
use sea_orm::{
    ColumnTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait, QueryFilter,
    Statement,
};
use serde::{Deserialize, Serialize};
use session::TokenSession;

#[derive(Clone)]
struct DbConnection {
    db_connection: sea_orm::DatabaseConnection,
}

// struct TSession {
//     token_session: Mutex<dyn TokenSession>
// }

fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(hello_world);

    let user_scope = web::scope("/user")
        .service(user_create)
        .service(user_exists)
        .service(user_login_token)
        .service(user_data);
    cfg.service(user_scope);
}

async fn setup_database(db_url: &str, db_name: &str) -> Result<DatabaseConnection, DbErr> {
    use sea_orm_migration::prelude::*;

    let db = Database::connect(db_url)
        .await
        .expect("failed to connecto to database");

    let schema_manager = SchemaManager::new(&db);

    use sea_orm::DatabaseBackend;
    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
            ))
            .await?;
            let url = format!("{}/{}", db_url, db_name);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            // if schema_manager.has("login_data").await? {
            //     db.execute(Statement::from_string(
            //         db.get_database_backend(),
            //         format!("CREATE DATABASE \"{}\";", DATABASE_NAME),
            //     ))
            //     .await?;
            // }
            // let url = format!("{}/{}", DATABASE_URL, DATABASE_NAME);
            // Database::connect(&url).await?
            panic!("postgresql not supported")
        }
        DbBackend::Sqlite => db,
    };

    migrator::Migrator::up(&db, None).await?;

    Ok(db)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let args = run_args::RunArgs::parse();

    log!(Level::Debug, "{:?}", args);

    let secret_key = Key::generate();
    let db = setup_database(&args.db, &args.db_name)
        .await
        .unwrap_or_else(|e| panic!("database setup error: {}", e));

    let data = DbConnection {
        db_connection: db.clone(),
    };

    use std::sync::Arc;

    let token_session: Arc<Mutex<dyn TokenSession>> =
        Arc::new(Mutex::new(session::DefaultTokenSession::default()));

    let token_session = web::Data::from(token_session);

    let _server = HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .configure(configure_services)
            .app_data(web::Data::new(data.clone()))
            .app_data(token_session.clone())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(Logger::default())
            .wrap(cors)
    })
    .bind((args.adress, args.port))?
    .run()
    .await;

    Ok(())
}

#[actix_web::get("/")]
async fn hello_world() -> impl Responder {
    "hello world"
}

#[derive(Deserialize, Debug)]
struct UserCreationData {
    login: String,
    password: String,
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
            };

            let res = match LoginData::insert(active).exec(db).await {
                Ok(_) => HttpResponse::Created().finish(),
                Err(db_err) => {
                    log!(
                        Level::Error,
                        "database insert error at create user for login: '{}' , err: '{:?}'",
                        creation.login,
                        db_err
                    );
                    HttpResponse::InternalServerError().finish()
                }
            };
            res
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

#[derive(Deserialize, Serialize)]
struct UserLogin {
    login: String,
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
                "user_egsists error for login '{}' err: {:?}",
                user_login.login,
                err
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize)]
struct LoginToken {
    token: String,
}

impl Responder for LoginToken {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(actix_web::http::header::ContentType::json())
            .body(body)
    }
}

#[actix_web::post("/token")]
async fn user_login_token(
    login_data: web::Json<UserCreationData>,
    data: web::Data<DbConnection>,
    token_session: web::Data<Mutex<dyn TokenSession>>,
    session: Session,
) -> Result<HttpResponse, errors::TokenError> {
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
    match sess.get_user(&uuid) {
        Some(usr) => {
            log!(Level::Debug, "session for {}", usr);
            let db = &data.db_connection;
            let usr = LoginData::find()
                .filter(login_data::Column::Login.eq(&usr))
                .one(db)
                .await
                .unwrap()
                .unwrap();
            let usr_data = UserLogin { login: usr.login };
            return HttpResponse::Ok()
                .reason("session exists")
                .body(serde_json::to_string(&usr_data).unwrap());
        }
        None => {
            log!(Level::Debug, "no session for token: {}", uuid);
            HttpResponse::Forbidden()
                .reason("no such user session")
                .finish()
        }
    }
}
