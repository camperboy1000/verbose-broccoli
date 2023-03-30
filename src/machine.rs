use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder,
};
use sqlx::query_as;

use crate::models::{AppState, Machine};

#[get("/")]
async fn get_all_machines(data: Data<AppState>) -> impl Responder {
    match query_as::<_, Machine>("SELECT * FROM machine")
        .fetch_all(&data.database)
        .await
    {
        Ok(machines) => HttpResponse::Ok().json(machines),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/{machine_id}")]
async fn get_machine(data: Data<AppState>, path: Path<i32>) -> impl Responder {
    let machine_id = path.into_inner();

    match query_as::<_, Machine>("SELECT * FROM machine WHERE id = $1")
        .bind(machine_id)
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
