use crate::http::Result;
use crate::models::game::{GameRecord, PlayerColor};
use crate::models::rooms_manager::RoomsManagerTrait;
use crate::repositories::game_repository::GameRepositoryTrait;
use uuid::Uuid;

pub struct CreateGameService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> CreateGameService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, player_id: Uuid) -> Result<Uuid> {
        let player_color = PlayerColor::random();

        let game_record = GameRecord::new(player_id, player_color);

        let game_record_id = game_record.id.clone();

        let player = self.game_repository.get_player(player_id).await?;

        self.rooms_manager.create_room(game_record_id);
        self.rooms_manager
            .add_player(game_record_id, player, Some(player_color))
            .unwrap();

        self.game_repository.save_game(game_record).await?;

        Ok(game_record_id)
    }
}
