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
mod tests {
    use super::*;

    #[test]
    fn test_add_player_rooms_manager() {
        let rooms_manager = RoomsManager::_new_empty();

        let room_id = Uuid::new_v4();

        let player1 = Player {
            id: Uuid::new_v4(),
            username: String::from("Ding Liren"),
            email: String::from("ding@liren.com"),
        };

        let player2 = Player {
            id: Uuid::new_v4(),
            username: String::from("Elon Musk"),
            email: String::from("elon@musk.com"),
        };

        let p1_id = player1.id.clone();
        let p2_id = player2.id.clone();

        rooms_manager.create_room(room_id);

        let player1_color = rooms_manager.add_player(room_id, player1, None).unwrap();

        let room = rooms_manager.get_room(room_id).unwrap();

        assert!(room.white_player.is_some());
        assert!(room.black_player.is_none());
        assert!(player1_color == PlayerColor::White);
        assert_eq!(p1_id, room.white_player.clone().unwrap().id);

        let player2_color = rooms_manager.add_player(room_id, player2, None).unwrap();

        let room = rooms_manager.get_room(room_id).unwrap();

        assert!(room.white_player.is_some());
        assert!(room.black_player.is_some());
        assert!(player2_color == PlayerColor::Black);
        assert_eq!(p2_id, room.black_player.unwrap().id);
    }

    #[test]
    fn test_add_player_to_room() {
        let mut room = Room::new();

        let player1 = Player {
            id: Uuid::new_v4(),
            username: String::from("Ding Liren"),
            email: String::from("ding@liren.com"),
        };

        let player2 = Player {
            id: Uuid::new_v4(),
            username: String::from("Ding Liren"),
            email: String::from("ding@liren.com"),
        };

        let p1_id = player1.id.clone();
        let p2_id = player2.id.clone();

        let player1_color = room.add_player(player1, None).unwrap();

        assert!(room.white_player.is_some());
        assert!(room.black_player.is_none());
        assert!(player1_color == PlayerColor::White);
        assert_eq!(p1_id, room.white_player.clone().unwrap().id);

        let player2_color = room.add_player(player2, None).unwrap();

        assert!(room.white_player.is_some());
        assert!(room.black_player.is_some());
        assert!(player2_color == PlayerColor::Black);
        assert_eq!(p2_id, room.black_player.unwrap().id);
    }
}
