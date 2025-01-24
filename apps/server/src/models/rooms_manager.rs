use crate::http::Result;
use crate::models::game::GameRecord;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::broadcast;
use uuid::Uuid;

pub struct Player {
    id: Uuid,
    username: String,
    email: String,
}

pub enum ColorPlayer {
    WHITE,
    BLACK,
}

pub enum PairedGame {
    NewGame(GameRecord),
    ExistingGame(Uuid),
}

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

    fn notify_other_player(&self, username: String) {
        self.tx
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

    pub fn add_player(
        &mut self,
        player: Player,
        color_preference: Option<ColorPlayer>,
    ) -> Result<(), ()> {
        if self.is_full() {
            return Err(());
        }

        let username = player.username.clone();

        match color_preference {
            Some(ColorPlayer::WHITE) => {
                if self.white_player.is_some() {
                    return Err(());
                }
                self.white_player = Some(player);
            }
            Some(ColorPlayer::BLACK) => {
                if self.black_player.is_some() {
                    return Err(());
                }
                self.black_player = Some(player);
            }
            None => {
                if self.white_player.is_none() {
                    self.white_player = Some(player)
                } else {
                    self.black_player = Some(player)
                }
            }
        }

        if self.is_full() {
            self.notify_other_player(username);
        }

        Ok(())
    }
}

pub trait RoomsManagerTrait: Send + Sync {
    fn get_room(&self, room_id: Uuid) -> Option<broadcast::Sender<String>>;
    fn create_room(&self, room_id: Uuid) -> broadcast::Sender<String>;
    fn add_player(
        &self,
        room_id: Uuid,
        player: Player,
        color_preference: Option<ColorPlayer>,
    ) -> Result<(), ()>;
    fn pair_new_player(&self, player_id: Uuid) -> Result<PairedGame, ()>;
}

pub struct RoomsManager {
    pub game_rooms: Arc<Mutex<HashMap<Uuid, Room>>>,
    pub waiting_room: Arc<Mutex<Option<Uuid>>>,
    // pub waiting_rooms: Arc<Mutex<HashMap<String, Option<Uuid>>>>,
}

impl RoomsManagerTrait for RoomsManager {
    fn get_room(&self, room_id: Uuid) -> Option<broadcast::Sender<String>> {
        self.game_rooms
            .lock()
            .unwrap()
            .get(&room_id)
            .map(|room| room.tx.clone())
    }

    fn create_room(&self, room_id: Uuid) -> broadcast::Sender<String> {
        let (tx, _) = broadcast::channel(100);
        self.game_rooms.lock().unwrap().insert(
            room_id,
            Room {
                tx: tx.clone(),
                white_player: None,
                black_player: None,
            },
        );

        tx
    }

    fn add_player(
        &self,
        room_id: Uuid,
        player: Player,
        color_preference: Option<ColorPlayer>,
    ) -> Result<(), ()> {
        self.game_rooms
            .lock()
            .unwrap()
            .get_mut(&room_id)
            .map(|room| room.add_player(player, color_preference));

        Ok(())
    }

    fn pair_new_player(&self, player_id: Uuid) -> Result<PairedGame, ()> {
        let mut waiting_room = self.waiting_room.lock().unwrap();

        let (paired_game, new_waiting_room) = match waiting_room.take() {
            Some(room_id) => (PairedGame::ExistingGame(room_id), None),
            None => {
                let new_room_id = Uuid::new_v4();
                (
                    PairedGame::NewGame(GameRecord {
                        id: new_room_id,
                        white_player: Some(player_id),
                        ..Default::default()
                    }),
                    Some(new_room_id),
                )
            }
        };
        *waiting_room = new_waiting_room;

        Ok(paired_game)
    }
}
