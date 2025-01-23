use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::OnceCell;

pub static DB: OnceCell<SharedDB> = OnceCell::const_new();

pub type SharedDB = Arc<Pool<Postgres>>;

pub async fn init_db() {
    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL is void"))
        .await
        .expect("DB is not working");

    let db = Arc::new(db);
    DB.set(db).unwrap();
}

pub fn get_db() -> SharedDB {
    DB.get().expect("Database has not been initialized").clone()
}
