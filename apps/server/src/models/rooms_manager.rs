use super::event::Event;
use super::game::PlayerColor;
use crate::{http::Result, states::rooms_manager, Error};
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
    pub white_player: Option<Uuid>,
    pub black_player: Option<Uuid>,
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
        player_id: Uuid,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor> {
        if self.is_full() {
            return Err(Error::Conflict {
                message: String::from("Room is full!"),
            });
        }

        let player_color = match color_preference {
            Some(PlayerColor::White) | None if self.white_player.is_none() => {
                self.white_player = Some(player_id);
                Ok(PlayerColor::White)
            }
            Some(PlayerColor::Black) | None if self.black_player.is_none() => {
                self.black_player = Some(player_id);
                Ok(PlayerColor::Black)
            }
            _ => Err(Error::Conflict {
                message: String::from("A player already occupied you side!"),
            }),
        }?;

        if self.is_full() {
            self.relay_event(Event::Join);
        }

        Ok(player_color)
    }

    pub fn is_playing(&self, player_id: Uuid) -> bool {
        Some(player_id) == self.white_player || Some(player_id) == self.black_player
    }
}

#[automock]
pub trait RoomsManagerTrait: Send + Sync {
    fn get_room_tx(&self, room_id: Uuid) -> Result<broadcast::Sender<String>>;

    fn get_room(&self, room_id: Uuid) -> Result<Room>;
    fn create_room(&self, room_id: Uuid, request_key: String);
    fn add_player(
        &self,
        room_id: Uuid,
        player_id: Uuid,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor>;
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
    fn get_room_tx(&self, room_id: Uuid) -> Result<broadcast::Sender<String>> {
        Ok(self
            .game_rooms
            .lock()
            .unwrap()
            .get(&room_id)
            .ok_or(Error::NotFound {
                message: String::from("Room not found!"),
            })?
            .tx
            .clone())
    }

    fn get_room(&self, room_id: Uuid) -> Result<Room> {
        Ok(self
            .game_rooms
            .lock()
            .unwrap()
            .get_mut(&room_id)
            .ok_or(Error::NotFound {
                message: String::from("Room not found!"),
            })?
            .clone())
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
        player_id: Uuid,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor> {
        self.game_rooms
            .lock()
            .unwrap()
            .get_mut(&room_id)
            .ok_or(Error::NotFound {
                message: String::from("Room not found!"),
            })?
            .add_player(player_id, color_preference)
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
mod tests {
    use super::*;

    #[test]
    fn test_add_player_rooms_manager() {
        let rooms_manager = RoomsManager::_new_empty();

        let room_id = Uuid::new_v4();

        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();

        rooms_manager.create_room(room_id, String::from("123"));

        let player1_color = rooms_manager.add_player(room_id, player1, None);

        let room = rooms_manager.get_room(room_id).unwrap();

        assert!(room.white_player.is_some());
        assert!(room.black_player.is_none());
        assert!(player1_color.ok() == Some(PlayerColor::White));
        assert_eq!(Some(player1), room.white_player);

        let player2_color = rooms_manager.add_player(room_id, player2, None);

        let room = rooms_manager.get_room(room_id).unwrap();

        assert!(room.white_player.is_some());
        assert!(room.black_player.is_some());
        assert!(player2_color.ok() == Some(PlayerColor::Black));
        assert_eq!(Some(player2), room.black_player);
    }

    #[test]
    fn test_add_player_to_room() {
        let mut room = Room::new(String::from("123"));

        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();

        let player1_color = room.add_player(player1, None);

        assert!(room.white_player.is_some());
        assert!(room.black_player.is_none());
        assert!(player1_color.ok() == Some(PlayerColor::White));
        assert_eq!(Some(player1), room.white_player);

        let player2_color = room.add_player(player2, None);

        assert!(room.white_player.is_some());
        assert!(room.black_player.is_some());
        assert!(player2_color.ok() == Some(PlayerColor::Black));
        assert_eq!(Some(player2), room.black_player);
    }

    #[test]
    fn test_add_2_players_to_room() {
        let mut room = Room::new(String::from(""));

        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();

        let player1_color = room.add_player(player1, Some(PlayerColor::Black));

        assert!(player1_color.is_ok());
        assert!(room.black_player.is_some());
        assert!(room.white_player.is_none());
        assert_eq!(player1_color.ok(), Some(PlayerColor::Black));

        let player2_color = room.add_player(player2, None);

        assert!(player2_color.is_ok());
        assert!(room.white_player.is_some());
        assert!(room.black_player.is_some());
        assert!(player2_color.ok() == Some(PlayerColor::White));
    }

    #[test]
    fn test_two_players_picking_white() {
        let mut room = Room::new(String::from(""));

        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();

        let player1_color = room.add_player(player1, Some(PlayerColor::White));

        assert!(player1_color.is_ok());
        assert!(room.white_player.is_some());
        assert!(room.black_player.is_none());
        assert_eq!(player1_color.ok(), Some(PlayerColor::White));

        let player2_color = room.add_player(player2, Some(PlayerColor::White));

        assert!(player2_color.is_err());
        assert!(room.white_player.is_some());
        assert!(room.black_player.is_none());
    }

    #[test]
    fn test_add_three_players() {
        let mut room = Room::new(String::from(""));

        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();
        let player3 = Uuid::new_v4();

        let player1_color = room.add_player(player1, Some(PlayerColor::White));

        assert!(player1_color.is_ok());
        assert!(room.white_player.is_some());
        assert!(room.black_player.is_none());
        assert_eq!(player1_color.ok(), Some(PlayerColor::White));

        let player2_color = room.add_player(player2, None);

        assert!(player2_color.is_ok());
        assert!(room.white_player.is_some());
        assert!(room.black_player.is_some());
        assert_eq!(player2_color.ok(), Some(PlayerColor::Black));

        let player3_color = room.add_player(player3, None);

        assert!(player3_color.is_err());
        assert!(room.white_player.is_some());
        assert!(room.black_player.is_some());
    }
}
