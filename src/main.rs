use std::{env, io};

use actix_web::{web, App, HttpServer};
use log::{error, info, LevelFilter};
use sqlx::PgPool;

fn initalize_syslog() {
    let log_level: LevelFilter = match env::var("LOG_LEVEL") {
        Err(_) => log::LevelFilter::Warn,
        Ok(value) => match value.to_uppercase().as_str() {
            "ERROR" => log::LevelFilter::Error,
            "WARNING" => log::LevelFilter::Warn,
            "INFO" => log::LevelFilter::Info,
            "DEBUG" => log::LevelFilter::Debug,
            "TRACE" => log::LevelFilter::Trace,
            "OFF" => log::LevelFilter::Off,
            _ => log::LevelFilter::Warn,
        },
    };
    let log_result = syslog::init(syslog::Facility::LOG_SYSLOG, log_level, None);
    if log_result.is_err() {
        eprintln!("WARNING! Failed to initialize logging system! Server logs will be unavaliable!");
    }
}

fn parse_database_url() -> String {
    match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(err) => {
            let message = format!("ERROR: Unable to parse DATABASE_URL enviroment variable: {err}");
            error!("{message}");
            eprintln!("{message}");
            panic!("{err}");
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    initalize_syslog();

    let database_url = parse_database_url();

    let pool = match PgPool::connect_lazy(database_url.as_str()) {
        Ok(pool) => {
            info!("Connected to the database");
            pool
        }
        Err(err) => {
            let message = format!("ERROR: Failed to connect to the database: {err}");
            error!("{message}");
            eprintln!("{message}");
            panic!("{err}");
        }
    };

    HttpServer::new(|| App::new().service(web::scope("/user")))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
