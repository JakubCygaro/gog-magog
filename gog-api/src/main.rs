mod args;
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
use actix_web::{self, cookie::Key, dev::Server, middleware::Logger, web, App, HttpServer};
use log::{log, Level};
use sea_orm::{Database, DatabaseConnection, DbBackend, DbErr, Statement};
use service::DbConnection;
use session::TokenSession;

fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(service::hello_world);

    let user_scope = web::scope("/user")
        .service(service::user_create)
        .service(service::user_exists)
        .service(service::user_login_token)
        .service(service::user_data);
    cfg.service(user_scope);
}

async fn setup_database(db_url: &str, db_name: &str) -> Result<DatabaseConnection, DbErr> {
    use sea_orm_migration::prelude::*;

    let db = Database::connect(db_url)
        .await
        .expect("failed to connecto to database");

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

    migrator::Migrator::up(&db, None).await?;

    Ok(db)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let args = run_args::RunArgs::parse();

    log!(Level::Debug, "{:?}", args);

    create_and_run_server(&args).await?.await?;
    Ok(())
}

async fn create_and_run_server(args: &RunArgs) -> std::io::Result<Server> {
    let secret_key = Key::generate();
    let db = setup_database(&args.db, &args.db_name)
        .await
        .unwrap_or_else(|e| panic!("database setup error: {}", e));

    let data = DbConnection::new(db.clone());
    use std::sync::Arc;

    let token_session: Arc<Mutex<dyn TokenSession>> =
        Arc::new(Mutex::new(session::DefaultTokenSession::default()));

    let token_session = web::Data::from(token_session);

    Ok(HttpServer::new(move || {
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
    .bind((args.adress.clone(), args.port))?
    .run())
}
