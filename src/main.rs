use std::{env, process};

use actix_web::{web, App, HttpServer};
use laundry_api::{
    machine,
    models::{AppState, Machine, MachineType, Report, ReportType, Room, User},
    report::{self, ReportSubmission},
    room, user,
};
use sqlx::{PgPool, Pool, Postgres};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

const APP_NAME: &str = "Laundry API";

/// Initialize the logging system, using [syslog] as the backend.
fn initalize_syslog() {
    let log_level = match env::var("LOG_LEVEL") {
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
            eprintln!("Unable to parse DATABASE_URL enviroment variable: {err}");
            process::exit(1);
        }
    };

    match PgPool::connect_lazy(database_url.as_str()) {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("Failed to connect to the database: {err}");
            process::exit(1);
        }
    }
}

#[actix_web::main]
async fn main() {
    initalize_syslog();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            machine::get_all_machines,
            machine::get_machine,
            room::get_all_rooms,
            room::get_room,
            user::get_all_users,
            user::get_user,
            report::get_all_reports,
            report::get_report,
            report::submit_report,
            report::delete_report
        ),
        components(schemas(
            Machine,
            Room,
            Report,
            User,
            MachineType,
            ReportType,
            ReportSubmission
        ))
    )]
    struct ApiDoc;
    let openapi = ApiDoc::openapi();

    let app_state = AppState {
        database: connect_postgres_database(),
    };

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
            .service(
                web::scope("/user")
                    .service(user::get_all_users)
                    .service(user::get_user),
            )
            .service(
                web::scope("/report")
                    .service(report::get_all_reports)
                    .service(report::get_report)
                    .service(report::submit_report)
                    .service(report::delete_report),
            )
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-doc/openapi.json", openapi.clone()))
            .app_data(web::Data::new(app_state.clone()))
    });

    let http_server = match http_server.bind(("127.0.0.1", 8080)) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("ERROR! Failed to bind the webserver: {err}");
            process::exit(1);
        }
    };

    match http_server.run().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("ERROR! Gave up waiting for HttpServer to run: {err}");
            process::exit(1);
        }
    };
}
