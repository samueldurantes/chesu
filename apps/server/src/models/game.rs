#![allow(dead_code)]
use rand::random;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shakmaty::{san::San, Chess, Color, Outcome, Position};
use sqlx::prelude::FromRow;
use std::str::FromStr;

use uuid::Uuid;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerColor {
    White,
    Black,
}

impl PlayerColor {
    pub fn random() -> Self {
        if random::<i8>() % 2 == 0 {
            return Self::White;
        }

        Self::Black
    }

    pub fn choose(color_preference: Option<Self>) -> Self {
        match color_preference {
            Some(color) => color,
            None => PlayerColor::random(),
        }
    }

    pub fn to_string(self) -> String {
        match self {
            Self::White => String::from("white_player"),
            Self::Black => String::from("black_player"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, JsonSchema, FromRow, Default)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

impl Player {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            username: String::new(),
            email: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Default, Clone)]
pub struct Game {
    pub id: Uuid,
    pub white_player: Option<Player>,
    pub black_player: Option<Player>,
    pub bet_value: i32,
    pub state: GameState,
    pub moves: Vec<String>,
}

impl Game {
    pub fn to_game_record(self) -> GameRecord {
        GameRecord {
            id: self.id,
            white_player: self.white_player.map(|player| player.id),
            black_player: self.black_player.map(|player| player.id),
            state: self.state.to_string(),
            bet_value: self.bet_value,
            moves: self.moves,
        }
    }

    pub fn get_turn_color(&self) -> PlayerColor {
        if self.moves.len() % 2 == 0 {
            PlayerColor::White
        } else {
            PlayerColor::Black
        }
    }

    pub fn get_player_color(&self, player_id: Uuid) -> Option<PlayerColor> {
        if self.white_player.as_ref().map(|p| p.id) == Some(player_id) {
            return Some(PlayerColor::White);
        }

        if self.black_player.as_ref().map(|p| p.id) == Some(player_id) {
            return Some(PlayerColor::Black);
        }

        None
    }

    pub fn check_move(&self, mv: String) -> Result<Option<GameState>, ()> {
        let mut position = Chess::default();

        let mut moves = self.moves.clone();
        moves.push(mv);

        for san_move in moves.iter() {
            let parsed_move = San::from_str(san_move)
                .map_err(|_| ())?
                .to_move(&position)
                .map_err(|_| ())?;

            position = position.play(&parsed_move).map_err(|_| ())?;
        }

        let new_game_state = match position.outcome() {
            Some(Outcome::Decisive { winner }) => match winner {
                Color::White => Some(GameState::WhiteWin),
                Color::Black => Some(GameState::BlackWin),
            },
            Some(Outcome::Draw) => Some(GameState::Draw),
            None => None,
        }
        .unwrap_or_else(|| {
            if moves.len() >= 2 {
                GameState::Running
            } else {
                GameState::Waiting
            }
        });

        if self.state == new_game_state {
            return Ok(None);
        }

        Ok(Some(new_game_state))
    }
}

#[derive(Default, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
pub enum GameState {
    #[default]
    Waiting,
    Running,
    Draw,
    WhiteWin,
    BlackWin,
}

impl GameState {
    pub fn to_string(self) -> String {
        let result = match self {
            Self::Waiting => "waiting",
            Self::Running => "running",
            Self::Draw => "draw",
            Self::WhiteWin => "white_win",
            Self::BlackWin => "black_win",
        };

        String::from(result)
    }

    pub fn from_str(state: &str) -> Self {
        match state {
            "running" => Self::Running,
            "draw" => Self::Draw,
            "white_win" => Self::WhiteWin,
            "black_win" => Self::BlackWin,
            _ => Self::Waiting,
        }
    }
}

#[derive(Default, Clone, FromRow)]
pub struct GameRecord {
    pub id: Uuid,
    pub white_player: Option<Uuid>,
    pub black_player: Option<Uuid>,
    pub state: String,
    pub bet_value: i32,
    pub moves: Vec<String>,
}

impl GameRecord {
    fn new_empty() -> Self {
        Self {
            id: Uuid::new_v4(),
            ..Default::default()
        }
    }

    pub fn new(player_id: Uuid, color_preference: PlayerColor) -> Self {
        let mut game_record = Self::new_empty();

        match color_preference {
            PlayerColor::White => game_record.white_player = Some(player_id),
            PlayerColor::Black => game_record.black_player = Some(player_id),
        };

        game_record
    }
}
