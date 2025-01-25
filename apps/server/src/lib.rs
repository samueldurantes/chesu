use http::game::Player;
use http::{Error, Result};
use sqlx::Pool;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

pub mod app;
pub mod http;
pub mod states;

mod models;
mod repositories;
mod routes;
mod services;

use sqlx::{PgPool, Postgres};

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

    pub fn add_player(&mut self, id: &str) {
        self.players.lock().unwrap().insert(id.to_string());
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub rooms: Arc<Mutex<HashMap<String, RoomState>>>,
    pub available_game: Arc<Mutex<Option<Uuid>>>,
}

pub enum PlayerInput {
    User(Player),
    Id(Uuid),
}

impl AppState {
    pub fn new(db: Arc<Pool<Postgres>>) -> Self {
        AppState {
            db,
            rooms: Arc::new(Mutex::new(HashMap::new())),
            available_game: Arc::new(Mutex::new(None::<Uuid>)),
        }
    }

    pub fn add_player_to_room(&self, room_id: Uuid, player: PlayerInput) -> Result<()> {
        let mut rooms = self.rooms.as_ref().lock().unwrap();
        let room = rooms
            .entry(room_id.to_string())
            .or_insert_with(RoomState::new);

        let mut room_players = room.players.lock().unwrap();

        if room_players.len() > 2 {
            return Err(Error::BadRequest {
                message: "Game is full".to_string(),
            });
        }

        match player {
            PlayerInput::User(player) => {
                room_players.insert(player.id.to_string());
                if room_players.len() == 2 {
                    notify_other_player(&room, player.username);
                }
            }

            PlayerInput::Id(player) => {
                room_players.insert(player.to_string());
                assert!(room_players.len() != 2)
            }
        }

        Ok(())
    }
}

fn notify_other_player(room: &RoomState, username: String) {
    room.tx
        .send(
            serde_json::json!({
               "event": "join",
               "data": {
                 "username": username
                }
            })
            .to_string(),
        )
        .expect("Error on notify other player!");
}
