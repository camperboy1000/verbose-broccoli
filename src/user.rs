use actix_web::{
    delete, get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Pool, Postgres};

use crate::models::{AppState, User};

#[derive(Serialize, Deserialize)]
struct UserSubmission {
    username: String,
    admin: bool,
}

async fn is_username_present(
    database: &Pool<Postgres>,
    username: &String,
) -> Result<bool, sqlx::Error> {
    match query!(
        r#"
        SELECT username
        FROM public.user
        WHERE username = $1
        "#,
        username
    )
    .fetch_optional(database)
    .await
    {
        Ok(username) => Ok(username.is_some()),
        Err(err) => Err(err),
    }
}

#[utoipa::path(
    context_path = "/user",
    responses(
        (status = 200, description = "Lists all users", body = Vec<User>),
        (status = 500, description = "An internal server error occurred")
    )
)]
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status = 200, description = "The requested user", body=User),
        (status = 404, description = "The requested user was not found"),
        (status = 500, description = "An internal server error occurred")
    )
)]
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status = 201, description = "The user requested", body = User),
        (status = 409, description = "The requested username is already in use"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[post("/")]
async fn add_user(
    data: Data<AppState>,
    Json(user_submission): Json<UserSubmission>,
) -> impl Responder {
    let username_present =
        match is_username_present(&data.database, &user_submission.username).await {
            Ok(result) => result,
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status = 200, description = "Delected the requested user", body = User),
        (status = 404, description = "The requested user was not found"),
        (status = 500, description = "An internal server error occurred")
    )
)]
#[delete("/{username}")]
async fn delete_user(data: Data<AppState>, path: Path<String>) -> impl Responder {
    let username = path.into_inner();

    let username_present = match is_username_present(&data.database, &username).await {
        Ok(result) => result,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    if !username_present {
        return HttpResponse::NotFound().body(format!("The user {username} was not found."));
    }

    match query_as!(
        User,
        r#"
        DELETE FROM public.user
        WHERE username = $1
        RETURNING username, admin
        "#,
        &username
    )
    .fetch_one(&data.database)
    .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
