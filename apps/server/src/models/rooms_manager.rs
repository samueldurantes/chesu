use super::event::Event;
use super::game::PlayerColor;
use crate::{http::Result, states::rooms_manager, Error};
use chrono::{DateTime, Utc};
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
    pub white_time: ChessTime,
    pub black_time: ChessTime,
    pub tx: broadcast::Sender<String>,
}

#[derive(Debug, Clone)]
pub struct ChessTime {
    pub rest: i32,
    pub last_move: Option<DateTime<Utc>>,
    pub additional_time: i32,
}

impl ChessTime {
    pub fn new(time: i32, additional_time: i32) -> ChessTime {
        Self {
            rest: time * 60,
            last_move: None,
            additional_time,
        }
    }

    pub fn handle_move(&mut self) -> Result<i32, String> {
        let now = Utc::now();

        if let Some(last) = self.last_move {
            self.rest -= (now - last).num_seconds() as i32;
        }

        if self.rest < 0 {
            return Err(String::from("Your time is gone!"));
        }

        self.last_move = Some(now);

        self.rest += self.additional_time;

        Ok(self.rest)
    }
}

impl Room {
    pub fn new(request_key: String, time: i32, additional_time: i32) -> Self {
        Self {
            request_key,
            white_player: None,
            black_player: None,
            white_time: ChessTime::new(time, additional_time),
            black_time: ChessTime::new(time, additional_time),
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

pub struct CreateRoomInfo {
    pub room_id: Uuid,
    pub request_key: String,
    pub time: i32,
    pub additional_time: i32,
}

#[automock]
pub trait RoomsManagerTrait: Send + Sync {
    fn get_room_tx(&self, room_id: Uuid) -> Result<broadcast::Sender<String>>;
    fn get_room(&self, room_id: Uuid) -> Result<Room>;
    fn create_room(&self, info: CreateRoomInfo);
    fn handle_move_time(&self, room_id: Uuid, player_id: Uuid) -> Result<i32, String>;
    fn add_player(
        &self,
        room_id: Uuid,
        player_id: Uuid,
        color_preference: Option<PlayerColor>,
    ) -> Result<PlayerColor>;
    fn pair_new_player(&self, room_key: &str) -> PairedGame;
    fn remove_request(&self, request_key: &str);
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

    fn create_room(&self, info: CreateRoomInfo) {
        self.game_rooms.lock().unwrap().insert(
            info.room_id,
            Room::new(info.request_key, info.time, info.additional_time),
        );
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

    fn pair_new_player(&self, key: &str) -> PairedGame {
        let mut requests = self.requests.lock().unwrap();

        match requests.remove(key) {
            Some(room_id) => PairedGame::ExistingGame(room_id),
            None => {
                let room_id = Uuid::new_v4();
                requests.insert(key.to_string(), room_id);

                PairedGame::NewGame(room_id)
            }
        }
    }

    fn remove_request(&self, request_key: &str) {
        self.requests.lock().unwrap().remove(request_key);
    }

    fn remove_room(&self, room_id: Uuid) {
        self.game_rooms.lock().unwrap().remove(&room_id);
    }

    fn handle_move_time(&self, room_id: Uuid, player_id: Uuid) -> Result<i32, String> {
        let room = self.get_room(room_id)?;

        tracing::info!("Actual Time, {:#?}", room);

        match Some(player_id) {
            p if p == room.white_player => self
                .game_rooms
                .lock()
                .unwrap()
                .get_mut(&room_id)
                .ok_or(Error::NotFound {
                    message: String::from("Room not found!"),
                })?
                .white_time
                .handle_move(),
            p if p == room.black_player => self
                .game_rooms
                .lock()
                .unwrap()
                .get_mut(&room_id)
                .ok_or(Error::NotFound {
                    message: String::from("Room not found!"),
                })?
                .black_time
                .handle_move(),
            _ => Err(String::from("You are not playing!")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl RoomsManager {
        pub fn new_empty() -> Self {
            Self {
                game_rooms: Arc::new(Mutex::new(HashMap::new())),
                requests: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[test]
    fn test_add_player_rooms_manager() {
        let rooms_manager = RoomsManager::new_empty();

        let room_id = Uuid::new_v4();

        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();

        rooms_manager.create_room(CreateRoomInfo {
            room_id,
            request_key: String::from("w-10-0-0"),
            time: 10,
            additional_time: 0,
        });

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
        let mut room = Room::new(String::from("w-10-0-0"), 10, 0);

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
        let mut room = Room::new(String::from("w-10-0-0"), 10, 0);

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
        let mut room = Room::new(String::from("w-10-0-0"), 10, 0);

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
        let mut room = Room::new(String::from("w-10-0-0"), 10, 0);

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

    #[test]
    fn test_pairing_new_room() {
        let rooms_manager = RoomsManager::new_empty();
        let result = rooms_manager.pair_new_player("w-10-0-0");

        if let PairedGame::ExistingGame(_) = result {
            panic!()
        }
    }

    #[test]
    fn test_pairing_2_not_equal_rooms() {
        let rooms_manager = RoomsManager::new_empty();
        rooms_manager.pair_new_player("w-10-0-1");
        let result = rooms_manager.pair_new_player("w-10-0-0");

        if let PairedGame::ExistingGame(_) = result {
            panic!()
        }
    }

    #[test]
    fn test_pairing_existing_room() {
        let rooms_manager = RoomsManager::new_empty();
        let player = Uuid::new_v4();

        if let PairedGame::NewGame(room_id) = rooms_manager.pair_new_player("w-10-0-0") {
            rooms_manager.create_room(CreateRoomInfo {
                room_id,
                time: 10,
                additional_time: 0,
                request_key: String::from("w-10-0-0"),
            });
            rooms_manager
                .add_player(room_id, player, Some(PlayerColor::Black))
                .unwrap();
        }

        let result = rooms_manager.pair_new_player("w-10-0-0");

        match result {
            PairedGame::ExistingGame(room_id) => {
                let room = rooms_manager.get_room(room_id).unwrap();

                assert_eq!(room.request_key, "w-10-0-0");
                assert_eq!(room.black_player, Some(player));
            }
            _ => panic!(),
        }
    }
}
