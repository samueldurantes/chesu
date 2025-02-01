use uuid::Uuid;

use crate::http::{Error, Result};
use crate::models::game::Player;
use crate::models::rooms_manager::RoomsManagerTrait;
use crate::repositories::game_repository::GameRepositoryTrait;
use crate::repositories::game_repository::GameWithPlayers;

pub struct GetGameService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

fn get_mocked_player() -> Player {
    Player {
        id: Uuid::new_v4(),
        username: String::from("Waiting player..."),
        email: String::new(),
    }
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> GetGameService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, room_id: Uuid) -> Result<GameWithPlayers> {
        let room = self.rooms_manager.get_room(room_id)?;

        if !room.is_full() {
            let (white_player, black_player) = match (room.white_player, room.black_player) {
                (Some(player_id), None) => Ok((
                    self.game_repository.get_player(player_id).await?,
                    get_mocked_player(),
                )),
                (None, Some(player_id)) => Ok((
                    get_mocked_player(),
                    self.game_repository.get_player(player_id).await?,
                )),
                _ => Err(Error::InternalServerError),
            }?;

            return Ok(GameWithPlayers {
                id: room_id,
                white_player,
                black_player,
                ..Default::default()
            });
        }

        Ok(self.game_repository.get_game_with_players(room_id).await?)
    }
}
