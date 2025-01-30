pub mod db {
    use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
    use tokio::sync::OnceCell;

    static DB: OnceCell<Pool<Postgres>> = OnceCell::const_new();

    pub async fn init() {
        let db = PgPoolOptions::new()
            .max_connections(50)
            .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL is void"))
            .await
            .expect("DB is not working");

        DB.set(db).unwrap();
    }

    pub fn get() -> Pool<Postgres> {
        DB.get().expect("Database has not been initialized").clone()
    }
}

pub mod rooms_manager {
    use crate::models::rooms_manager::{GameRooms, Requests};
    use std::sync::Mutex;
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::OnceCell;

    static GAME_ROOMS: OnceCell<GameRooms> = OnceCell::const_new();
    static REQUESTS: OnceCell<Requests> = OnceCell::const_new();

    fn init() {
        GAME_ROOMS
            .set(Arc::new(Mutex::new(HashMap::new())))
            .unwrap();
        REQUESTS.set(Arc::new(Mutex::new(HashMap::new()))).unwrap();
    }

    fn get_rooms_manager() -> (GameRooms, Requests) {
        let game_rooms = GAME_ROOMS
            .get()
            .expect("Game rooms has not been initialized")
            .clone();

        let requests = REQUESTS
            .get()
            .expect("Waiting rooms has not been initialized")
            .clone();

        (game_rooms, requests)
    }

    pub fn get() -> (GameRooms, Requests) {
        if GAME_ROOMS.get().is_none() || REQUESTS.get().is_none() {
            init();
        }

        get_rooms_manager()
    }
}
