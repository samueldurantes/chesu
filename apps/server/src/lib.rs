use sqlx::PgPool;

pub mod app;
pub mod http;

#[derive(Clone)]
pub struct State {
    pub db: PgPool,
}
