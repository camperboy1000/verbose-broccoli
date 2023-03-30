use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse, Responder,
};
use sqlx::query_as;

use crate::models::{AppState, Report};

#[get("/")]
async fn get_all_reports(data: Data<AppState>) -> impl Responder {
    match query_as::<_, Report>("SELECT * FROM report")
        .fetch_all(&data.database)
        .await
    {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/{report_id}")]
async fn get_report(data: Data<AppState>, path: Path<i32>) -> impl Responder {
    let report_id = path.into_inner();

    match query_as::<_, Report>("SELECT * FROM report WHERE id = $1")
        .bind(report_id)
        .fetch_one(&data.database)
        .await
    {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
