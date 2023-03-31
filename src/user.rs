use actix_web::{
    get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};

use crate::models::{AppState, User};

#[derive(Serialize, Deserialize)]
struct UserSubmission {
    username: String,
    admin: bool,
}

#[get("/")]
async fn get_all_users(data: Data<AppState>) -> impl Responder {
    match query_as!(
        User,
        r#"
        SELECT username, admin
        FROM public.user
        "#
    )
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

    match query_as!(
        User,
        r#"
        SELECT username, admin
        FROM public.user
        WHERE username = $1
        "#,
        username
    )
    .fetch_optional(&data.database)
    .await
    {
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Ok(user) => match user {
            Some(user) => HttpResponse::Ok().json(&user),
            None => HttpResponse::NotFound().finish(),
        },
    }
}

#[post("/")]
async fn create_user(
    data: Data<AppState>,
    Json(user_submission): Json<UserSubmission>,
) -> impl Responder {
    let username_present = match query!(
        r#"
        SELECT username
        FROM public.user
        WHERE username = $1
        "#,
        &user_submission.username
    )
    .fetch_optional(&data.database)
    .await
    {
        Ok(username) => username.is_some(),
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    if username_present {
        return HttpResponse::Conflict()
            .body(format!("{} is already a user", &user_submission.username));
    }

    match query_as!(
        User,
        r#"
        INSERT INTO public.user (username, admin)
        VALUES ($1, $2)
        RETURNING username, admin
        "#,
        &user_submission.username,
        &user_submission.admin
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(user) => HttpResponse::Created().json(user),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
