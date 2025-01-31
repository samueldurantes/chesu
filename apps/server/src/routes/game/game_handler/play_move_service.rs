use crate::http::Result;
use crate::models::event::MoveInfo;
use crate::models::game::GameState;
use crate::repositories::game_repository::GameRepositoryTrait;

pub struct PlayMoveService<R: GameRepositoryTrait> {
    game_repository: R,
}

impl<R: GameRepositoryTrait> PlayMoveService<R> {
    pub fn new(game_repository: R) -> Self {
        Self { game_repository }
    }

    pub async fn execute(
        &self,
        MoveInfo {
            game_id,
            player_id,
            move_played,
        }: MoveInfo,
    ) -> Result<(), String> {
        let game = self
            .game_repository
            .get_game(game_id)
            .await
            .map_err(|_| String::from("Game not found!"))?;

        match (game.get_player_color(player_id), game.get_turn_color()) {
            (None, _) => Err(String::from("You are not playing this game!")),
            (Some(player_color), turn_color) if player_color == turn_color => Ok(()),
            (_, _) => Err(String::from("It's not your turn!")),
        }?;

        if game.moves.len() == 1 {
            self.game_repository
                .update_state(game_id, GameState::Running)
                .await
                .map_err(|_| String::from("Move record failed!"))?;
        }

        self.game_repository
            .record_move(game_id, move_played)
            .await
            .map_err(|_| String::from("Move record failed!"))?;

        Ok(())
    }
}
