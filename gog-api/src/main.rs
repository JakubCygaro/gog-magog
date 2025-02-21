#![allow(clippy::all)]

mod args;
mod cache;
mod entity;
mod errors;
mod migrator;
mod service;
mod session;
use actix_cors::Cors;
use args::{self as run_args, RunArgs};
use clap::Parser;
use std::sync::Mutex;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{self, cookie::Key, dev::Server, guard, middleware::Logger, web, App, HttpServer};
use log::{log, Level};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbBackend, DbErr, Statement};
use service::DbConnection;
use session::TokenSession;

fn configure_services(cfg: &mut web::ServiceConfig) {
    use service::resources;
    cfg.service(service::hello_world);

    let user_scope = web::scope("/user")
        .service(service::user_create)
        .service(service::user_exists)
        .service(service::user_login_token)
        .service(service::user_data)
        .service(service::user_update)
        .service(service::user_logout)
        .service(
            web::resource("/upload_pfp")
                .guard(guard::Header("content-type", "image/jpg"))
                .guard(guard::Post())
                .route(web::post().to(resources::user_upload_pfp)),
        )
        .service(service::user_get_pfp)
        // .service(service::user_profile_name)
        // .service(service::user_profile_id)
        .service(service::user_profile);
    cfg.service(user_scope);
    let posts_scope = web::scope("/posts")
        .service(service::posts::posts_create)
        .service(service::posts::posts_newest)
        .service(service::posts::posts_user)
        .service(service::posts::posts_filter)
        .service(service::posts::posts_id);
    cfg.service(posts_scope);
}

async fn setup_database(
    db_url: &str,
    db_name: &str,
    refresh: bool,
) -> Result<DatabaseConnection, DbErr> {
    use sea_orm_migration::prelude::*;

    let mut c_opt = ConnectOptions::new(db_url);
    c_opt.sqlx_logging(false);

    let db = Database::connect(c_opt)
        .await
        .expect("failed to connect to database");

    let _schema_manager = SchemaManager::new(&db);

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
    if refresh {
        migrator::Migrator::fresh(&db).await?;
    }
    migrator::Migrator::up(&db, None).await?;

    Ok(db)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let args = run_args::RunArgs::parse();

    log!(
        Level::Info,
        "Running gog-magog server on {}:{}\nwith database url: {} and database name: {}",
        &args.adress,
        &args.port,
        &args.db,
        &args.db_name
    );
    create_and_run_server(&args).await?.await?;
    Ok(())
}

async fn create_and_run_server(args: &RunArgs) -> std::io::Result<Server> {
    let secret_key = Key::generate();
    let db = setup_database(&args.db, &args.db_name, args.fresh)
        .await
        .unwrap_or_else(|e| panic!("database setup error: {}", e));

    let db = DbConnection::new(db.clone());
    use std::sync::Arc;

    let token_session: Arc<Mutex<dyn TokenSession>> =
        Arc::new(Mutex::new(session::DefaultTokenSession::new(Some(600))));

    let token_session = web::Data::from(token_session);

    let cache = Arc::new(Mutex::new(cache::ResourceCache::new()));
    let cache = web::Data::from(cache);
    Ok(HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .configure(configure_services)
            .app_data(web::Data::new(db.clone()))
            .app_data(token_session.clone())
            .app_data(cache.clone())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(Logger::default())
            .wrap(cors)
    })
    .bind((args.adress.clone(), args.port))?
    .run())
}
