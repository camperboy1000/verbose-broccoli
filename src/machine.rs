use actix_web::{
    delete, get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use utoipa::ToSchema;

use crate::{
    models::{AppState, Machine, MachineType, Report, ReportType},
    room,
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MachineSubmission {
    room_id: i32,
    machine_id: String,
    machine_type: MachineType,
}

pub async fn is_machine_present(
    database: &Pool<Postgres>,
    room_id: &i32,
    machine_id: &String,
) -> Result<bool, sqlx::Error> {
    match query!(
        r#"
        SELECT room_id, machine_id
        FROM machine
        WHERE room_id = $1
            AND machine_id = $2
        "#,
        room_id,
        machine_id
    )
    .fetch_optional(database)
    .await
    {
        Ok(result) => Ok(result.is_some()),
        Err(err) => Err(err),
    }
}

#[utoipa::path(
    context_path = "/machine",
    responses(
        (status = 200, description = "List of all machines", body = Vec<Machine>, example = json!([{
            "room_id": 1,
            "machine_id": "A",
            "machine_type": "Dryer"
        }])),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[get("/")]
async fn get_all_machines(data: Data<AppState>) -> impl Responder {
    match query_as!(
        Machine,
        r#"
        SELECT
            room_id,
            machine_id,
            type as "machine_type: MachineType"
        FROM machine
        "#,
    )
    .fetch_all(&data.database)
    .await
    {
        Ok(machines) => HttpResponse::Ok().json(machines),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/machine",
    responses(
        (status = 200, description = "The requested machine", body = Machine, example = json!({
            "room_id": 1,
            "machine_id": "A",
            "machine_type": "Dryer"
        })),
        (status = 404, description = "The requested machine was not found"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[get("/{room_id}/{machine_id}")]
async fn get_machine(data: Data<AppState>, path: Path<(i32, String)>) -> impl Responder {
    let (room_id, machine_id) = path.into_inner();

    match query_as!(
        Machine,
        r#"
        SELECT
            room_id,
            machine_id,
            type as "machine_type: MachineType"
        FROM machine
        WHERE room_id = $1
            AND machine_id = $2
        "#,
        room_id,
        machine_id
    )
    .fetch_optional(&data.database)
    .await
    {
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        Ok(machine) => match machine {
            Some(machine) => HttpResponse::Ok().json(&machine),
            None => HttpResponse::NotFound().json(format!(
                "Machine id {machine_id} was not found in room id {room_id}."
            )),
        },
    }
}

#[utoipa::path(
    context_path = "/machine",
    request_body(content = MachineSubmission, content_type = "application/json", example = json!({
        "room_id": 1,
        "machine_id": "A",
        "machine_type": "Dryer"
    })),
    responses(
        (status = 201, description = "The requested machine was created", body = Machine, example = json!({
            "room_id": 1,
            "machine_id": "A",
            "machine_type": "Dryer"
        })),
        (status = 400, description = "The requested room does not exist"),
        (status = 409, description = "The requested machine already exists"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[post("/")]
async fn add_machine(
    data: Data<AppState>,
    Json(machine_submission): Json<MachineSubmission>,
) -> impl Responder {
    let room_present =
        match room::is_room_present(&data.database, &machine_submission.room_id).await {
            Ok(result) => result,
            Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
        };

    if !room_present {
        return HttpResponse::BadRequest().json(format!(
            "The room id {} was not found.",
            &machine_submission.room_id
        ));
    }

    let machine_present = match is_machine_present(
        &data.database,
        &machine_submission.room_id,
        &machine_submission.machine_id,
    )
    .await
    {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if machine_present {
        return HttpResponse::Conflict().json(format!(
            "Machine id {} already exists in room id {}.",
            &machine_submission.machine_id, &machine_submission.room_id
        ));
    }

    match query_as!(
        Machine,
        r#"
        INSERT INTO machine (room_id, machine_id, type)
        VALUES ($1, $2, $3)
        RETURNING
            room_id,
            machine_id,
            type AS "machine_type: MachineType"
        "#,
        &machine_submission.room_id,
        &machine_submission.machine_id,
        &machine_submission.machine_type as &MachineType
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(machine) => HttpResponse::Created().json(machine),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/machine",
    responses(
        (status = 200, description = "The requested machine was deleted", body = Machine, example = json!({
            "room_id": 1,
            "machine_id": "A",
            "machine_type": "Dryer"
        })),
        (status = 404, description = "The requested machine was not found"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[delete("/{room_id}/{machine_id}")]
async fn delete_machine(data: Data<AppState>, path: Path<(i32, String)>) -> impl Responder {
    let (room_id, machine_id) = path.into_inner();

    let machine_present = match is_machine_present(&data.database, &room_id, &machine_id).await {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if !machine_present {
        return HttpResponse::NotFound().json(format!(
            "Machine id {machine_id} was not found in room id {room_id}."
        ));
    }

    match query_as!(
        Machine,
        r#"
        DELETE FROM machine
        WHERE room_id = $1
            AND machine_id = $2
        RETURNING
            room_id,
            machine_id,
            type AS "machine_type: MachineType"
        "#,
        &room_id,
        &machine_id
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(machine) => HttpResponse::Ok().json(machine),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/machine",
    responses(
        (status = 200, description = "List of all unarchived reports for the requested machine", body = Vec<Report>, example = json!([{
            "report_id": 1,
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
            "time": "2023-01-01T12:00:00.000Z",
            "archived": false,
        }])),
        (status = 400, description = "The requested query was invalid"),
        (status = 500, description = "An internal server occurred")
    )
)]
#[get("/{room_id}/{machine_id}/reports")]
async fn get_machine_reports(data: Data<AppState>, path: Path<(i32, String)>) -> impl Responder {
    let (room_id, machine_id) = path.into_inner();

    let machine_present = match is_machine_present(&data.database, &room_id, &machine_id).await {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if !machine_present {
        return HttpResponse::BadRequest().json(format!(
            "Machine id {machine_id} was not found in room id {room_id}"
        ));
    }

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
        WHERE room_id = $1
            AND machine_id = $2
            AND archived = false
        "#,
        &room_id,
        &machine_id
    )
    .fetch_all(&data.database)
    .await
    {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/machine",
    responses(
        (status = 200, description = "List of all unarchived reports for the requested machine", body = Vec<Report>, example = json!([{
            "report_id": 1,
            "room_id": 1,
            "machine_id": "A",
            "reporter_username": "admin",
            "report_type": "Broken",
            "description": "No heat",
            "time": "2023-01-01T12:00:00.000Z",
            "archived": true,
        }])),
        (status = 400, description = "The requested query was invalid"),
        (status = 500, description = "An internal server occurred")
    )
)]
#[get("/{room_id}/{machine_id}/reports/archived")]
async fn get_machine_archived_reports(
    data: Data<AppState>,
    path: Path<(i32, String)>,
) -> impl Responder {
    let (room_id, machine_id) = path.into_inner();

    let machine_present = match is_machine_present(&data.database, &room_id, &machine_id).await {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if !machine_present {
        return HttpResponse::BadRequest().json(format!(
            "Machine id {machine_id} was not found in room id {room_id}"
        ));
    }

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
        WHERE room_id = $1
            AND machine_id = $2
            AND archived = true
        "#,
        &room_id,
        &machine_id
    )
    .fetch_all(&data.database)
    .await
    {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
