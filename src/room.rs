use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder,
};
use sqlx::query_as;

use crate::models::{AppState, Room};

#[utoipa::path(
    context_path = "/room",
    responses(
        (status = 200, description = "Lists all rooms", body = Vec<Room>),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[get("/")]
async fn get_all_rooms(data: Data<AppState>) -> impl Responder {
    match query_as!(
        Room,
        r#"
        SELECT id as "room_id: i32", name, description
        FROM room
        "#
    )
    .fetch_all(&data.database)
    .await
    {
        Ok(rooms) => HttpResponse::Ok().json(rooms),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/room",
    responses(
        (status = 200, description = "The requested room", body = Room),
        (status = 404, description = "The requested room was not found"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[get("/{room_id}")]
async fn get_room(data: Data<AppState>, path: Path<i32>) -> impl Responder {
    let room_id = path.into_inner();

    match query_as!(
        Room,
        r#"
        SELECT id as "room_id: i32", name, description
        FROM room
        WHERE id = $1
        "#,
        room_id
    )
    .fetch_optional(&data.database)
    .await
    {
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Ok(room) => match room {
            Some(room) => HttpResponse::Ok().json(&room),
            None => HttpResponse::NotFound().finish(),
        },
    }
}
