use sqlx::Pool;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

use sqlx::{PgPool, Postgres};

pub mod app;
pub mod http;

pub struct RoomState {
    pub players: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<String>,
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            players: Mutex::new(HashSet::new()),
            tx: broadcast::channel(100).0,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub rooms: Arc<Mutex<HashMap<String, RoomState>>>,
    pub available_game: Arc<Mutex<Option<Uuid>>>,
}

impl AppState {
    pub fn new(db: Pool<Postgres>) -> Self {
        AppState {
            db,
            rooms: Arc::new(Mutex::new(HashMap::new())),
            available_game: Arc::new(Mutex::new(None::<Uuid>)),
        }
    }
}
