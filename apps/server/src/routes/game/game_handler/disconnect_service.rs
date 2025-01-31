use crate::models::{
    event::{DisconnectInfo, Event},
    game::GameState,
};
use crate::repositories::game_repository::GameRepositoryTrait;
use crate::{http::Result, models::rooms_manager::RoomsManagerTrait};

pub struct DisconnectService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> DisconnectService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, info: DisconnectInfo) -> Result<(), String> {
        let room = self
            .rooms_manager
            .get_room(info.game_id)
            .ok_or(String::from("Room not found!"))?;

        if !room.is_full() {
            self.rooms_manager.remove_request(room.request_key);
            self.rooms_manager.remove_room(info.game_id);
            return Ok(());
        }

        let game = self
            .game_repository
            .get_game(info.game_id)
            .await
            .map_err(|_| String::from("Game not found!"))?;

        let white_player = game.white_player.map(|p| p.id);
        let black_player = game.black_player.map(|p| p.id);

        let new_game_state = match (game.state, info.player_id) {
            (GameState::Waiting, _) => Some(GameState::Draw),
            (GameState::Running, player_id) if Some(player_id) == white_player => {
                Some(GameState::BlackWin)
            }
            (GameState::Running, player_id) if Some(player_id) == black_player => {
                Some(GameState::WhiteWin)
            }
            _ => None,
        };

        if let Some(new_game_state) = new_game_state {
            self.game_repository
                .update_state(game.id, new_game_state.clone())
                .await
                .map_err(|m| m.to_string())?;

            room.relay_event(Event::GameChangeState(new_game_state));
        }

        self.rooms_manager.remove_room(info.game_id);

        Ok(())
    }
}
