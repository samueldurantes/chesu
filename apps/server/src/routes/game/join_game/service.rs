use crate::http::Result;
use crate::models::rooms_manager::RoomsManagerTrait;
use crate::repositories::game_repository::GameRepositoryTrait;
use uuid::Uuid;

pub struct JoinGameService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> JoinGameService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, player_id: Uuid, room_id: Uuid) -> Result<Uuid> {
        let player = self.game_repository.get_player(player_id).await?;

        let player_color = self
            .rooms_manager
            .add_player(room_id, player, None)
            .unwrap();

        self.game_repository
            .add_player(room_id, player_id, player_color)
            .await?;

        Ok(room_id)
    }
}
