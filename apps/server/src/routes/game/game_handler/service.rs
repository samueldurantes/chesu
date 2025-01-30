use crate::http::Result;
use crate::repositories::game_repository::GameRepositoryTrait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoveInfo {
    pub game_id: Uuid,
    pub player_id: Uuid,
    pub move_played: String,
}

impl MoveInfo {
    pub fn from_str(str: &String) -> Result<Self, String> {
        serde_json::from_str(str).map_err(|_| String::from("Failed to build the move"))
    }
}

pub struct PlayMoveService<R: GameRepositoryTrait> {
    game_repository: R,
}

impl<R: GameRepositoryTrait> PlayMoveService<R> {
    pub fn new(game_repository: R) -> Self {
        Self { game_repository }
    }

    pub async fn execute(&self, move_info: MoveInfo) -> Result<(), String> {
        let game = self
            .game_repository
            .get_game(move_info.game_id)
            .await
            .map_err(|_| String::from("Game not found!"))?;

        match (
            game.get_player_color(move_info.player_id),
            game.get_turn_color(),
        ) {
            (None, _) => Err(String::from("You are not playing this game!")),
            (Some(player_color), turn_color) if player_color == turn_color => Ok(()),
            (_, _) => Err(String::from("It's not your turn!")),
        }?;

        self.game_repository
            .record_move(move_info.game_id, move_info.move_played)
            .await
            .map_err(|_| String::from("Move record failed!"))?;

        Ok(())
    }
}
