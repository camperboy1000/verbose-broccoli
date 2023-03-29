use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder,
};
use sqlx::query_as;

use crate::models::{AppState, User};

#[get("/")]
async fn get_all_users(data: Data<AppState>) -> impl Responder {
    match query_as::<_, User>("SELECT * FROM public.user")
        .fetch_all(&data.database)
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/{username}")]
async fn get_user(data: Data<AppState>, path: Path<String>) -> impl Responder {
    let username = path.into_inner();

    match query_as::<_, User>("SELECT * FROM public.user WHERE username = $1")
        .bind(username)
        .fetch_one(&data.database)
        .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
