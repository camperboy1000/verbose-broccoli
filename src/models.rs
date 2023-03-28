use sqlx::{Pool, Postgres};

pub struct AppState {
    pub database: Pool<Postgres>,
}
