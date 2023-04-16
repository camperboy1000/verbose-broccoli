use actix_web::{
    delete, get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use time::{OffsetDateTime, PrimitiveDateTime};
use utoipa::ToSchema;

use crate::{
    machine,
    models::{AppState, Report, ReportType},
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReportSubmission {
    machine_id: String,
    room_id: i32,
    reporter_username: String,
    report_type: ReportType,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ArchiveSubmission {
    report_id: i32,
}

async fn is_report_present(
    database: &Pool<Postgres>,
    report_id: &i32,
) -> Result<bool, sqlx::Error> {
    match query!(
        r#"
        SELECT id
        FROM report
        WHERE id = $1
        "#,
        report_id
    )
    .fetch_optional(database)
    .await
    {
        Ok(result) => Ok(result.is_some()),
        Err(err) => Err(err),
    }
}

#[utoipa::path(
    context_path = "/report",
    responses(
        (status = 200, description = "List of all unarchived reports", body = Vec<Report>, example = json!([{
            "report_id": 1,
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
            "time": "2023-01-01T12:00:00.000Z",
            "archived": false,
          }])),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[get("/")]
async fn get_all_reports(data: Data<AppState>) -> impl Responder {
    match query_as!(
        Report,
        r#"
        SELECT 
            id AS "report_id: i32",
            room_id,
            machine_id,
            reporter_username,
            time,
            type AS "report_type: ReportType",
            description,
            archived
        FROM report
        WHERE archived = false
        "#,
    )
    .fetch_all(&data.database)
    .await
    {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/report",
    responses(
        (status = 200, description = "List of all archived reports", body = Vec<Report>, example = json!([{
            "report_id": 1,
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
            "time": "2023-01-01T12:00:00.000Z",
            "archived": true,
          }])),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[get("/archived")]
async fn get_all_archived_reports(data: Data<AppState>) -> impl Responder {
    match query_as!(
        Report,
        r#"
        SELECT 
            id AS "report_id: i32",
            room_id,
            machine_id,
            reporter_username,
            time,
            type AS "report_type: ReportType",
            description,
            archived
        FROM report
        WHERE archived = true
        "#,
    )
    .fetch_all(&data.database)
    .await
    {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/report",
    responses(
        (status = 200, description = "The requested report", body = Report, example = json!({
            "report_id": 1,
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
            "time": "2023-01-01T12:00:00.000Z",
            "archived": false,
          })),
        (status = 404, description = "The requested report was not found"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[get("/{report_id}")]
async fn get_report(data: Data<AppState>, path: Path<i32>) -> impl Responder {
    let report_id = path.into_inner();

    match query_as!(
        Report,
        r#"
        SELECT
            id AS "report_id: i32",
            room_id,
            machine_id,
            reporter_username,
            time,
            type AS "report_type: ReportType",
            description,
            archived
        FROM report
        WHERE id = $1
        "#,
        report_id
    )
    .fetch_optional(&data.database)
    .await
    {
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        Ok(report) => match report {
            Some(report) => HttpResponse::Ok().json(&report),
            None => {
                HttpResponse::NotFound().json(format!("The report id {report_id} was not found."))
            }
        },
    }
}

#[utoipa::path(
    context_path = "/report",
    request_body(
        content = ReportSubmission,
        content_type = "application/json",
        description = "JSON object containing the room id, machine id, reporter's username, report type, and an optional description",
        example = json!({
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
          })
    ),
    responses(
        (status = 201, description = "The report was created", body = Report, example = json!({
            "report_id": 1,
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
            "time": "2023-01-01T12:00:00.000Z",
            "archived": false,
          })),
        (status = 400, description = "The requested query was invalid"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[post("/")]
async fn submit_report(
    data: Data<AppState>,
    Json(report_submission): Json<ReportSubmission>,
) -> impl Responder {
    let machine_present = match machine::is_machine_present(
        &data.database,
        &report_submission.room_id,
        &report_submission.machine_id,
    )
    .await
    {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if !machine_present {
        return HttpResponse::BadRequest().json(format!(
            "Room id {} does not contain machine id {}.",
            &report_submission.room_id, &report_submission.machine_id
        ));
    }

    let current_time = OffsetDateTime::now_utc();

    match query_as!(
        Report,
        r#"
        INSERT INTO report (room_id, machine_id, reporter_username, type, description, time)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id AS "report_id: i32",
            room_id,
            machine_id,
            reporter_username,
            time,
            type AS "report_type: ReportType",
            description,
            archived
        "#,
        &report_submission.room_id,
        &report_submission.machine_id,
        &report_submission.reporter_username,
        &report_submission.report_type as &ReportType,
        report_submission.description,
        PrimitiveDateTime::new(current_time.date(), current_time.time())
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(report) => HttpResponse::Created().json(report),
        Err(err) => match err {
            sqlx::Error::Database(err) => HttpResponse::BadRequest().json(err.to_string()),
            _ => HttpResponse::InternalServerError().json(err.to_string()),
        },
    }
}

#[utoipa::path(
    context_path = "/report",
    responses(
        (status = 200, description = "The requested report was deleted", body = Report, example = json!({
            "report_id": 1,
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
            "time": "2023-01-01T12:00:00.000Z",
            "archived": false,
          })),
        (status = 404, description = "The requested report was not found"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[delete("/{report_id}")]
async fn delete_report(data: Data<AppState>, path: Path<i32>) -> impl Responder {
    let report_id = path.into_inner();

    let report_present = match is_report_present(&data.database, &report_id).await {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if !report_present {
        return HttpResponse::NotFound().json(format!("Report id {report_id} was not found."));
    }

    match query_as!(
        Report,
        r#"
    DELETE FROM report
    WHERE id = $1
    RETURNING
        id as "report_id: i32",
        room_id,
        machine_id,
        reporter_username,
        time,
        type as "report_type: ReportType",
        description,
        archived
    "#,
        report_id
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/report",
    request_body(
        content = ArchiveSubmission,
        content_type = "application/json",
        description = "JSON object containing the report id",
        example = json!({
            "report_id": 1,
        })
    ),
    responses(
        (status = 200, description = "The requested report was archived", body = Report, example = json!({
            "report_id": 1,
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
            "time": "2023-01-01T12:00:00.000Z",
            "archived": true,
        })),
        (status = 400, description = "The requested query was invalid"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[post("/archive")]
async fn archive_report(
    data: Data<AppState>,
    Json(archive_submission): Json<ArchiveSubmission>,
) -> impl Responder {
    let report_present =
        match is_report_present(&data.database, &archive_submission.report_id).await {
            Ok(result) => result,
            Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
        };

    if !report_present {
        return HttpResponse::BadRequest().json(format!(
            "Report id {} was not found.",
            &archive_submission.report_id
        ));
    }

    match query_as!(
        Report,
        r#"
        UPDATE report
        SET archived = true
        WHERE id = $1
        RETURNING
            id as "report_id: i32",
            room_id,
            machine_id,
            reporter_username,
            time,
            type as "report_type: ReportType",
            description,
            archived
        "#,
        &archive_submission.report_id
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
