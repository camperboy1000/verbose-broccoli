use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres, Type};

pub struct AppState {
    pub database: Pool<Postgres>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Machine {
    #[sqlx(rename = "id")]
    pub machine_id: String,
    pub room_id: i32,
    #[sqlx(rename = "type")]
    pub machine_type: MachineType,
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
    pub room_id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub username: String,
    pub admin: bool,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Report {
    #[sqlx(rename = "id")]
    pub report_id: i32,
    pub room_id: i32,
    pub machine_id: String,
    pub reporter_username: String,
    #[sqlx(rename = "type")]
    pub report_type: ReportType,
    pub time: NaiveDateTime,
    pub description: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "report_type", rename_all = "lowercase")]
pub enum ReportType {
    Operational,
    Caution,
    Broken,
}
