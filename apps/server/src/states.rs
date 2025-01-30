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
    use crate::models::rooms_manager::{GameRooms, WaitingRoom, WaitingRooms};
    use std::sync::Mutex;
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::OnceCell;

    static GAME_ROOMS: OnceCell<GameRooms> = OnceCell::const_new();
    static WAITING_ROOM: OnceCell<WaitingRoom> = OnceCell::const_new();
    static WAITING_ROOMS: OnceCell<WaitingRooms> = OnceCell::const_new();

    fn init() {
        GAME_ROOMS
            .set(Arc::new(Mutex::new(HashMap::new())))
            .unwrap();
        WAITING_ROOM.set(Arc::new(Mutex::new(None))).unwrap();
        WAITING_ROOMS
            .set(Arc::new(Mutex::new(HashMap::new())))
            .unwrap();
    }

    fn get_rooms_manager() -> (GameRooms, WaitingRoom, WaitingRooms) {
        let game_rooms = GAME_ROOMS
            .get()
            .expect("Game rooms has not been initialized")
            .clone();

        let waiting_room = WAITING_ROOM
            .get()
            .expect("Waiting room has not been initialized")
            .clone();

        let waitings_rooms = WAITING_ROOMS
            .get()
            .expect("Waiting rooms has not been initialized")
            .clone();

        (game_rooms, waiting_room, waitings_rooms)
    }

    pub fn get() -> (GameRooms, WaitingRoom, WaitingRooms) {
        if GAME_ROOMS.get().is_none() || WAITING_ROOM.get().is_none() {
            init();
        }

        get_rooms_manager()
    }
}
