#![allow(clippy::all)]

mod cache;
mod entity;
mod errors;
mod migrator;
mod service;
mod session;
use actix_cors::Cors;
use std::sync::Mutex;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{self, cookie::Key, dev::Server, middleware::Logger, web, App, HttpServer};
use log::{log, Level};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbBackend, DbErr, Statement};
use service::DbConnection;
use session::TokenSession;

fn configure_services(cfg: &mut web::ServiceConfig) {
    service::configure_service(cfg);
    service::posts::configure_service(cfg);
    service::comments::configure_service(cfg);
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

    dotenvy::dotenv().map_err(|_e| {
        log::warn!(".env file not present");
    }).unwrap_or_default();

    let args = clap::Command::new("gog-magog-api")
        // .version(clap::crate_version!())
        .arg(
            clap::Arg::new("address")
                .env(gog_commons::vars::BACKEND_ADDRESS_ENV)
                .short('a')
                .long("address"),
        )
        .arg(
            clap::Arg::new("port")
                .env(gog_commons::vars::BACKEND_PORT_ENV)
                .short('p')
                .long("port"),
        )
        .arg(
            clap::Arg::new("db")
                .env(gog_commons::vars::BACKEND_DATABASE_URL_ENV)
                .long("database-url"),
        )
        .arg(
            clap::Arg::new("db_name")
                .env(gog_commons::vars::BACKEND_DATABASE_NAME_ENV)
                .long("database-name"),
        )
        .arg(
            clap::Arg::new("fresh")
                .long("fresh")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();
    use gog_commons::vars::defaults;
    let address = args.get_one::<&str>("address").map_or(defaults::BACKEND_ADDRESS, |a| &a);
    let port = args.get_one::<u16>("port").map_or(defaults::BACKEND_PORT, |p| *p);
    let db = args.get_one::<String>("db").expect("db expected");
    let db_name = args.get_one::<String>("db_name").expect("db_name expected");
    log!(
        Level::Info,
        "Running gog-magog server on {}:{}\nwith database url: {} and database name: {}",
        address,
        port,
        db,
        db_name
    );
    create_and_run_server(
        &address,
        port,
        &db,
        &db_name,
        args.get_flag("fresh")
    )
    .await?
    .await?;
    Ok(())
}

async fn create_and_run_server(
    address: &str,
    port: u16,
    db: &str,
    db_name: &str,
    fresh: bool,
) -> std::io::Result<Server> {
    let secret_key = Key::generate();
    let db = setup_database(db, db_name, fresh)
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
    .bind((address, port))?
    .run())
}
