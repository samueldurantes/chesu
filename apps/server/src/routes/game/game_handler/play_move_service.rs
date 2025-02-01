use crate::http::{Error, Result};
use crate::models::event::Event;
use crate::models::event::MoveInfo;
use crate::models::game::GameState;
use crate::models::rooms_manager::RoomsManagerTrait;
use crate::repositories::game_repository::GameRepositoryTrait;

pub struct PlayMoveService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> PlayMoveService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(
        &self,
        MoveInfo {
            game_id,
            player_id,
            move_played,
        }: MoveInfo,
    ) -> Result<(), String> {
        let game = self.game_repository.get_game(game_id).await?;

        if game.get_player_color(player_id)? != game.get_turn_color() {
            return Err(Error::BadRequest {
                message: String::from("It's not your turn!"),
            }
            .to_string());
        }

        let game_state = game.check_move(move_played.clone())?;

        if let Some(new_game_state) = game_state {
            self.game_repository
                .update_state(game_id, new_game_state.clone())
                .await?;

            match new_game_state {
                GameState::Running => {}
                new_game_state => {
                    let room = self.rooms_manager.get_room(game_id)?;

                    room.relay_event(Event::GameChangeState(new_game_state));
                }
            }
        }

        self.game_repository
            .record_move(game_id, move_played)
            .await?;

        Ok(())
    }
}
