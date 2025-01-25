use uuid::Uuid;

pub enum ColorPlayer {
    WHITE,
    BLACK,
}

impl ColorPlayer {
    pub fn to_string(self) -> String {
        match self {
            Self::WHITE => String::from("white_player"),
            Self::BLACK => String::from("black_player"),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Default)]
pub struct Game {
    pub id: Uuid,
    pub white_player: Option<Player>,
    pub black_player: Option<Player>,
    pub bet_value: i32,
    pub moves: Vec<String>,
}

impl Game {
    pub fn new_empty() -> Self {
        Self {
            id: Uuid::new_v4(),
            ..Default::default()
        }
    }

    pub fn to_game_record(self) -> GameRecord {
        GameRecord {
            id: self.id,
            white_player: self.white_player.map(|player| player.id),
            black_player: self.black_player.map(|player| player.id),
            bet_value: self.bet_value,
            moves: self.moves,
        }
    }
}

#[derive(Default)]
pub struct GameRecord {
    pub id: Uuid,
    pub white_player: Option<Uuid>,
    pub black_player: Option<Uuid>,
    pub bet_value: i32,
    pub moves: Vec<String>,
}
