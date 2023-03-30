use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder,
};
use sqlx::query_as;

use crate::models::{AppState, Room};

#[get("/")]
async fn get_all_rooms(data: Data<AppState>) -> impl Responder {
    match query_as::<_, Room>("SELECT * FROM room")
        .fetch_all(&data.database)
        .await
    {
        Ok(rooms) => HttpResponse::Ok().json(rooms),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/{room_id}")]
async fn get_room(data: Data<AppState>, path: Path<i32>) -> impl Responder {
    let room_id = path.into_inner();

    match query_as::<_, Room>("SELECT * FROM room WHERE id = $1")
        .bind(room_id)
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
