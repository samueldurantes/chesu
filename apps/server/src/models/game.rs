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

#[derive(Serialize, Deserialize, JsonSchema, Default, Clone, FromRow)]
pub struct Game {
    pub id: Uuid,
    pub white_player: Uuid,
    pub black_player: Uuid,
    pub bet_value: i32,
    pub state: GameState,
    pub moves: Vec<String>,
}

impl Game {
    pub fn get_turn_color(&self) -> PlayerColor {
        match self.moves.len() % 2 {
            0 => PlayerColor::White,
            _ => PlayerColor::Black,
        }
    }

    pub fn get_player_color(&self, player_id: Uuid) -> Option<PlayerColor> {
        match player_id {
            player if self.white_player == player => Some(PlayerColor::White),
            player if self.black_player == player => Some(PlayerColor::Black),
            _ => None,
        }
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
    #[serde(rename = "waiting")]
    #[default]
    Waiting,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "draw")]
    Draw,
    #[serde(rename = "white_win")]
    WhiteWin,
    #[serde(rename = "black_win")]
    BlackWin,
}
