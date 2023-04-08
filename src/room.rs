use actix_web::{
    delete, get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};
use utoipa::ToSchema;

use crate::models::{AppState, Room};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RoomSubmission {
    name: String,
    description: Option<String>,
}

pub async fn is_room_present(
    database: &Pool<Postgres>,
    room_id: &i32,
) -> Result<bool, sqlx::Error> {
    match query!(
        r#"
        SELECT id
        FROM room
        WHERE id = $1
        "#,
        room_id
    )
    .fetch_optional(database)
    .await
    {
        Ok(result) => Ok(result.is_some()),
        Err(err) => Err(err),
    }
}

#[utoipa::path(
    context_path = "/room",
    responses(
        (status = 200, description = "Lists all rooms", body = Vec<Room>, example = json!([{
            "room_id": 1,
            "name": "Room 1",
            "description": "Room 1 in Complex A"
        }])),
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
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/room",
    responses(
        (status = 200, description = "The requested room", body = Room, example = json!({
            "room_id": 1,
            "name": "Room 1",
            "description": "Room 1 in Complex A"
        })),
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
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        Ok(room) => match room {
            Some(room) => HttpResponse::Ok().json(&room),
            None => HttpResponse::NotFound().json(format!("The room id {room_id} was not found.")),
        },
    }
}

#[utoipa::path(
    context_path = "/room",
    request_body(content = RoomSubmission, content_type = "application/json", example = json!({
        "name": "Room 1",
        "description": "Room 1 in Complex A"
    })),
    responses(
        (status = 201, description = "The requested room was created", body = Room, example = json!({
            "room_id": 1,
            "name": "Room 1",
            "description": "Room 1 in Complex A"
        })),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[post("/")]
async fn add_room(
    data: Data<AppState>,
    Json(room_submission): Json<RoomSubmission>,
) -> impl Responder {
    match query_as!(
        Room,
        r#"
        INSERT INTO room (name, description)
        VALUES ($1, $2)
        RETURNING
            id AS "room_id: i32",
            name,
            description
        "#,
        &room_submission.name,
        room_submission.description
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(room) => HttpResponse::Created().json(room),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[utoipa::path(
    context_path = "/room",
    responses(
        (status = 200, description = "The requested room was deleted", body = Room, example = json!({
            "room_id": 1,
            "name": "Room 1",
            "description": "Room 1 in Complex A"
        })),
        (status = 404, description = "The requested room was not found"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[delete("/{room_id}")]
async fn delete_room(data: Data<AppState>, path: Path<i32>) -> impl Responder {
    let room_id = path.into_inner();

    let room_present = match is_room_present(&data.database, &room_id).await {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if !room_present {
        return HttpResponse::NotFound().json(format!("Room id {room_id} was not found."));
    }

    match query_as!(
        Room,
        r#"
        DELETE FROM room
        WHERE id = $1
        RETURNING
            id AS "room_id: i32",
            name,
            description
        "#,
        &room_id
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(room) => HttpResponse::Ok().json(room),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
