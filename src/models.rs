use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres, Type};

pub struct AppState {
    pub database: Pool<Postgres>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Machine {
    #[sqlx(rename = "id")]
    machine_id: i32,
    #[sqlx(rename = "type")]
    machine_type: MachineType,
    room_id: i32,
}

#[derive(Serialize, Deserialize, Type)]
#[sqlx(type_name = "machine_type", rename_all = "lowercase")]
pub enum MachineType {
    Washer,
    Dryer,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Room {
    #[sqlx(rename = "id")]
    room_id: i32,
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    username: String,
    admin: bool,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Report {
    #[sqlx(rename = "id")]
    report_id: i32,
    machine_id: i32,
    reporter_username: String,
    #[sqlx(rename = "type")]
    report_type: ReportType,
    time: NaiveDateTime,
    description: Option<String>,
    archived: bool,
}

#[derive(Serialize, Deserialize, Type)]
#[sqlx(type_name = "report_type", rename_all = "lowercase")]
pub enum ReportType {
    Operational,
    Caution,
    Broken,
}
