#![allow(dead_code)]
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
    pub white_player: Option<Player>,
    pub black_player: Option<Player>,
    pub tx: broadcast::Sender<String>,
}

impl Room {
    pub fn new() -> Self {
        Self {
            white_player: None,
            black_player: None,
            tx: broadcast::channel(100).0,
        }
    }

    fn is_full(&self) -> bool {
        self.white_player.is_some() && self.black_player.is_some()
    }

    // fn notify_other_player(&self, username: String) {
    //     self.tx
    //         .send(
    //             serde_json::json!({
    //                "event": "join",
    //                "data": {
    //                  "username": username
    //                 }
    //             })
    //             .to_string(),
    //         )
    //         .expect("Error on notify other player!");
    // }

    pub fn add_player(
        &mut self,
        player: Player,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor, ()> {
        if self.is_full() {
            return Err(());
        }

        // let username = player.username.clone();

        let player_color = match color_preference {
            Some(PlayerColor::White) => {
                if self.white_player.is_some() {
                    return Err(());
                }
                self.white_player = Some(player);
                PlayerColor::White
            }
            Some(PlayerColor::Black) => {
                if self.black_player.is_some() {
                    return Err(());
                }
                self.black_player = Some(player);
                PlayerColor::Black
            }
            None => {
                if self.white_player.is_none() {
                    self.white_player = Some(player);
                    PlayerColor::White
                } else {
                    self.black_player = Some(player);
                    PlayerColor::Black
                }
            }
        };

        // if self.is_full() {
        //     self.notify_other_player(username);
        // }

        Ok(player_color)
    }
}

#[automock]
pub trait RoomsManagerTrait: Send + Sync {
    fn get_room_tx(&self, room_id: Uuid) -> Option<broadcast::Sender<String>>;

    fn get_room(&self, room_id: Uuid) -> Option<Room>;
    fn create_room(&self, room_id: Uuid);
    fn add_player(
        &self,
        room_id: Uuid,
        player: Player,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor, ()>;
    fn pair_new_player(&self) -> PairedGame;
}

pub type GameRooms = Arc<Mutex<HashMap<Uuid, Room>>>;
pub type WaitingRoom = Arc<Mutex<Option<Uuid>>>;

#[derive(Debug)]
pub struct RoomsManager {
    game_rooms: GameRooms,
    waiting_room: WaitingRoom,
    // pub waiting_rooms: Arc<Mutex<HashMap<String, Option<Uuid>>>>,
}

impl RoomsManager {
    pub fn new() -> Self {
        let (game_rooms, waiting_room) = rooms_manager::get();

        Self {
            game_rooms,
            waiting_room,
        }
    }

    pub fn _new_empty() -> Self {
        Self {
            game_rooms: Arc::new(Mutex::new(HashMap::new())),
            waiting_room: Arc::new(Mutex::new(None)),
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

    fn create_room(&self, room_id: Uuid) {
        self.game_rooms.lock().unwrap().insert(room_id, Room::new());
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

    fn pair_new_player(&self) -> PairedGame {
        let mut waiting_room = self.waiting_room.lock().unwrap();

        let (paired_game, new_waiting_room) = match waiting_room.take() {
            Some(room_id) => (PairedGame::ExistingGame(room_id), None),
            None => {
                let new_room_id = Uuid::new_v4();
                (PairedGame::NewGame(new_room_id), Some(new_room_id))
            }
        };
        *waiting_room = new_waiting_room;

        paired_game
    }
}

#[cfg(test)]
mod tests;
