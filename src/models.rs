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

#[derive(Type, Serialize, Deserialize)]
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
