use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

use sqlx::PgPool;

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
