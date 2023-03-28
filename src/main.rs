use std::{env, process};

use actix_web::{web, App, HttpServer};
use log::{error, info, LevelFilter};
use sqlx::PgPool;

static APP_NAME: &str = "Laundry-API";

/// Initialize the logging system, using [syslog] as the backend.
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

    if syslog::init(syslog::Facility::LOG_SYSLOG, log_level, Some(APP_NAME)).is_err() {
        eprintln!("WARNING: Failed to initialize logging system! Server logs will be unavaliable!");
    }
}

/// Parses and returns the database URL from the DATABASE_URL [environment variable](std::env::var).
///
/// # Exits
/// The DATABASE_URL environment variable  not being set is considered an unrecoverable error and exits the process.
fn parse_database_url() -> String {
    match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(err) => {
            throw_error_message(format!(
                "Unable to parse DATABASE_URL enviroment variable: {err}"
            ));
            process::exit(1);
        }
    }
}

/// This function attempts to send an error message to the logger and also prints the message to STDERR.
///
/// The message is a [String] which is passed to the system log and STDERR.
fn throw_error_message(message: String) {
    error!("ERROR: {message}");
    eprintln!("ERROR: {message}\n");
}

#[actix_web::main]
async fn main() {
    initalize_syslog();

    let database_url = parse_database_url();

    let pool = match PgPool::connect_lazy(database_url.as_str()) {
        Ok(pool) => {
            info!("INFO: Connected to the database");
            pool
        }
        Err(err) => {
            throw_error_message(format!("Failed to connect to the database: {err}"));
            process::exit(1);
        }
    };

    let http_server = HttpServer::new(|| App::new().service(web::scope("/user")));

    let http_server = match http_server.bind(("127.0.0.1", 8080)) {
        Ok(server) => server,
        Err(err) => {
            throw_error_message(format!("Failed to bind the webserver: {err}"));
            process::exit(1);
        }
    };

    match http_server.run().await {
        Ok(_) => (),
        Err(err) => {
            throw_error_message(format!("Gave up waiting for HttpServer to run: {err}"));
            process::exit(1);
        }
    };
}
