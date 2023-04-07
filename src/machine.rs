use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder,
};
use sqlx::query_as;

use crate::models::{AppState, Machine, MachineType};

#[utoipa::path(
    context_path = "/machine",
    responses(
        (status = 200, description = "List of all machines", body = Vec<Machine>),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[get("/")]
async fn get_all_machines(data: Data<AppState>) -> impl Responder {
    match query_as!(
        Machine,
        r#"
        SELECT
            id as "machine_id: String",
            room_id,
            type as "machine_type: MachineType"
        FROM machine
        "#,
    )
    .fetch_all(&data.database)
    .await
    {
        Ok(machines) => HttpResponse::Ok().json(machines),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/machine",
    responses(
        (status = 200, description = "The requested machine", body = Machine),
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
            id as "machine_id: String",
            room_id,
            type as "machine_type: MachineType"
        FROM machine
        WHERE id = $1
        AND room_id = $2
        "#,
        machine_id,
        room_id
    )
    .fetch_optional(&data.database)
    .await
    {
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Ok(machine) => match machine {
            Some(machine) => HttpResponse::Ok().json(&machine),
            None => HttpResponse::NotFound().finish(),
        },
    }
}
