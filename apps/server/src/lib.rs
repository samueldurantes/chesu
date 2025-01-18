use crate::http::game::Game;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::Notify;

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

pub struct PairingRoom {
    pub game: Mutex<Option<Game>>,
    pub notifier: Notify,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub rooms: Arc<Mutex<HashMap<String, RoomState>>>,
    pub pairing_room: Arc<PairingRoom>,
}
