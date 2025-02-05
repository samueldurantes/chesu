#![allow(dead_code)]
use crate::http::{Error, Result};
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
    pub time: i32,
    pub additional_time: i32,
    pub state: GameState,
    pub moves: Vec<String>,
}

fn invalid_move() -> Error {
    Error::BadRequest {
        message: String::from("Invalid move!"),
    }
}

impl Game {
    pub fn get_turn_color(&self) -> PlayerColor {
        match self.moves.len() % 2 {
            0 => PlayerColor::White,
            _ => PlayerColor::Black,
        }
    }

    pub fn get_player_color(&self, player_id: Uuid) -> Result<PlayerColor> {
        match player_id {
            player if self.white_player == player => Ok(PlayerColor::White),
            player if self.black_player == player => Ok(PlayerColor::Black),
            _ => Err(Error::BadRequest {
                message: String::from("You are not playing this game!"),
            }),
        }
    }

    pub fn check_move(&self, mv: &str) -> Result<Option<GameState>> {
        let mut position = Chess::default();

        let mut moves = self.moves.clone();
        moves.push(mv.to_string());

        for san_move in moves.iter() {
            let parsed_move = San::from_str(san_move)
                .map_err(|_| invalid_move())?
                .to_move(&position)
                .map_err(|_| invalid_move())?;

            position = position.play(&parsed_move).map_err(|_| invalid_move())?;
        }

        let new_game_state = match position.outcome() {
            Some(Outcome::Decisive {
                winner: Color::White,
            }) => GameState::WhiteWin,
            Some(Outcome::Decisive {
                winner: Color::Black,
            }) => GameState::BlackWin,
            Some(Outcome::Draw) => GameState::Draw,
            _ if moves.len() >= 2 => GameState::Running,
            _ => GameState::Waiting,
        };

        if self.state == new_game_state {
            return Ok(None);
        }

        Ok(Some(new_game_state))
    }
}

#[derive(Default, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug, Copy)]
pub enum GameState {
    #[default]
    Waiting,
    Running,
    Draw,
    WhiteWin,
    BlackWin,
}

impl GameState {
    pub fn from_str(input: &str) -> Result<Self> {
        match input {
            "waiting" => Ok(GameState::Waiting),
            "running" => Ok(GameState::Running),
            "draw" => Ok(GameState::Draw),
            "white_win" => Ok(GameState::WhiteWin),
            "black_win" => Ok(GameState::BlackWin),
            _ => Err(Error::InternalServerError),
        }
    }

    pub fn to_string(&self) -> String {
        let result = match self {
            GameState::Waiting => "waiting",
            GameState::Running => "running",
            GameState::Draw => "draw",
            GameState::WhiteWin => "white_win",
            GameState::BlackWin => "black_win",
        };

        result.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::models::game::{Game, GameState};
    use uuid::Uuid;

    impl Game {
        fn new() -> Self {
            Self {
                id: Uuid::new_v4(),
                white_player: Uuid::new_v4(),
                black_player: Uuid::new_v4(),
                ..Default::default()
            }
        }

        fn init_moves(&mut self, moves: Vec<&str>) {
            self.moves = moves.into_iter().map(|mv| mv.to_string()).collect();
        }
    }

    #[test]
    fn check_move_valid_move() {
        let mut game = Game::new();
        game.init_moves(vec!["e4"]);
        let new_game_state = game.check_move("c5").ok();

        assert_eq!(new_game_state, Some(Some(GameState::Running)))
    }

    #[test]
    fn check_move_not_new_game_state() {
        let mut game = Game::new();
        game.init_moves(vec!["e4", "c5"]);
        game.state = GameState::Running;

        let new_game_state = game.check_move("a3").ok();

        assert_eq!(new_game_state, Some(None))
    }

    #[test]
    fn check_move_invalid_move() {
        let mut game = Game::new();
        game.init_moves(vec!["e4", "c5", "a3"]);
        game.state = GameState::Running;

        let new_game_state = game.check_move("Ra7").ok();

        assert_eq!(new_game_state, None)
    }

    #[test]
    fn check_move_checkmate() {
        let mut game = Game::new();
        game.init_moves(vec!["e4", "e5", "Bc4", "a6", "Qf3", "a5"]);
        game.state = GameState::Running;

        let new_game_state = game.check_move("Qxf7#").ok();

        assert_eq!(new_game_state, Some(Some(GameState::WhiteWin)))
    }
    #[test]

    fn check_move_draw() {
        let mut game = Game::new();
        game.init_moves(vec![
            "e4", "d5", "exd5", "e6", "dxe6", "fxe6", "Nf3", "g5", "Nxg5", "Qxg5", "Nc3", "e5",
            "f4", "exf4", "d4", "f3", "gxf3", "Qg6", "Bd3", "Qg7", "f4", "Nc6", "Be3", "Bg4",
            "Be2", "Bxe2", "Qxe2", "O-O-O", "O-O-O", "Bb4", "Bd2", "Bxc3", "Bxc3", "Nf6", "d5",
            "Ne7", "Rhe1", "Rhe8", "Qe6+", "Kb8", "d6", "cxd6", "Rxd6", "Rxd6", "Qxd6+", "Ka8",
            "Bxf6", "Qf7", "Bxe7", "Rxe7", "Rxe7", "Qxe7", "Qxe7", "h5", "h4", "a5", "a4", "b5",
            "b3", "bxa4", "bxa4", "Kb8", "Qd7", "Ka8",
        ]);
        game.state = GameState::Running;

        let new_game_state = game.check_move("Qc7").ok();

        assert_eq!(new_game_state, Some(Some(GameState::Draw)))
    }
}
