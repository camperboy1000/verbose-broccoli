use std::{env, process};

use actix_web::{web, App, HttpServer};
use laundry_api::{machine, models::AppState, room};
use log::{error, info, LevelFilter};
use sqlx::{PgPool, Pool, Postgres};

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

/// Parses and returns a connection pool to the configured database.
/// The database URL is derived from the DATABASE_URL [environment variable](std::env::var).
///
/// # Exits
/// The DATABASE_URL environment variable not being set is considered an unrecoverable error which exits the process.
/// The process will also exit if an error occurs when attempting to connect to the database.
fn connect_postgres_database() -> Pool<Postgres> {
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(err) => {
            throw_error_message(format!(
                "Unable to parse DATABASE_URL enviroment variable: {err}"
            ));
            process::exit(1);
        }
    };

    match PgPool::connect_lazy(database_url.as_str()) {
        Ok(pool) => {
            info!("Connected to the database");
            pool
        }
        Err(err) => {
            throw_error_message(format!("Failed to connect to the database: {err}"));
            process::exit(1);
        }
    }
}

/// This function attempts to send an error message to the logger and also prints the message to STDERR.
///
/// The message is a [String] which is passed to the system log and STDERR.
fn throw_error_message(message: String) {
    error!("{message}");
    eprintln!("ERROR: {message}\n");
}

#[actix_web::main]
async fn main() {
    initalize_syslog();
    let database_pool = connect_postgres_database();

    let http_server = HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/machine")
                    .service(machine::get_all_machines)
                    .service(machine::get_machine),
            )
            .service(
                web::scope("/room")
                    .service(room::get_all_rooms)
                    .service(room::get_room),
            )
            .app_data(web::Data::new(AppState {
                database: database_pool.clone(),
            }))
    });

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
