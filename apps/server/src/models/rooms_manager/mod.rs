#![allow(dead_code)]
use super::event::Event;
use super::game::{Player, PlayerColor};
use crate::{http::Result, states::rooms_manager};
use mockall::automock;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use uuid::Uuid;

pub enum PairedGame {
    NewGame(Uuid),
    ExistingGame(Uuid),
}

#[derive(Debug, Clone)]
pub struct Room {
    pub request_key: String,
    pub white_player: Option<Player>,
    pub black_player: Option<Player>,
    pub tx: broadcast::Sender<String>,
}

impl Room {
    pub fn new(request_key: String) -> Self {
        Self {
            request_key,
            white_player: None,
            black_player: None,
            tx: broadcast::channel(100).0,
        }
    }

    pub fn is_full(&self) -> bool {
        self.white_player.is_some() && self.black_player.is_some()
    }

    pub fn relay_event(&self, event: Event) {
        self.tx.send(event.json()).unwrap_or(0);
    }

    pub fn add_player(
        &mut self,
        player: Player,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor, ()> {
        if self.is_full() {
            return Err(());
        }

        let player_color = match color_preference {
            Some(PlayerColor::White) | None if self.white_player.is_none() => {
                self.white_player = Some(player);
                Ok(PlayerColor::White)
            }
            Some(PlayerColor::Black) | None if self.black_player.is_none() => {
                self.black_player = Some(player);
                Ok(PlayerColor::Black)
            }
            _ => Err(()),
        }?;

        if self.is_full() {
            self.relay_event(Event::Join);
        }

        Ok(player_color)
    }
}

#[automock]
pub trait RoomsManagerTrait: Send + Sync {
    fn get_room_tx(&self, room_id: Uuid) -> Option<broadcast::Sender<String>>;

    fn get_room(&self, room_id: Uuid) -> Option<Room>;
    fn create_room(&self, room_id: Uuid, request_key: String);
    fn add_player(
        &self,
        room_id: Uuid,
        player: Player,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor, ()>;
    fn pair_new_player(&self, room_key: String) -> PairedGame;
    fn remove_request(&self, request_key: String);
    fn remove_room(&self, room_id: Uuid);
}

pub type GameRooms = Arc<Mutex<HashMap<Uuid, Room>>>;
pub type Requests = Arc<Mutex<HashMap<String, Uuid>>>;

#[derive(Debug)]
pub struct RoomsManager {
    game_rooms: GameRooms,
    requests: Requests,
}

impl RoomsManager {
    pub fn new() -> Self {
        let (game_rooms, requests) = rooms_manager::get();

        Self {
            game_rooms,
            requests,
        }
    }

    pub fn _new_empty() -> Self {
        Self {
            game_rooms: Arc::new(Mutex::new(HashMap::new())),
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl RoomsManagerTrait for RoomsManager {
    fn get_room_tx(&self, room_id: Uuid) -> Option<broadcast::Sender<String>> {
        self.game_rooms
            .lock()
            .unwrap()
            .get(&room_id)
            .map(|room| room.tx.clone())
    }

    fn get_room(&self, room_id: Uuid) -> Option<Room> {
        self.game_rooms
            .lock()
            .unwrap()
            .get_mut(&room_id)
            .map(|room| room.clone())
    }

    fn create_room(&self, room_id: Uuid, request_key: String) {
        self.game_rooms
            .lock()
            .unwrap()
            .insert(room_id, Room::new(request_key));
    }

    fn add_player(
        &self,
        room_id: Uuid,
        player: Player,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor, ()> {
        self.game_rooms
            .lock()
            .unwrap()
            .get_mut(&room_id)
            .map(|room| room.add_player(player, color_preference))
            .unwrap()
    }

    fn pair_new_player(&self, key: String) -> PairedGame {
        let mut waiting_rooms = self.requests.lock().unwrap();

        match waiting_rooms.remove(&key) {
            Some(room_id) => PairedGame::ExistingGame(room_id),
            None => {
                let room_id = Uuid::new_v4();
                waiting_rooms.insert(key, room_id);

                PairedGame::NewGame(room_id)
            }
        }
    }

    fn remove_request(&self, request_key: String) {
        self.requests.lock().unwrap().remove(&request_key);
    }

    fn remove_room(&self, room_id: Uuid) {
        self.game_rooms.lock().unwrap().remove(&room_id);
    }
}

#[cfg(test)]
mod tests;
