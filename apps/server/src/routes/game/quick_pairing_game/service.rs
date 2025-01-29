use crate::http::Result;
use crate::models::game::{GameRecord, PlayerColor};
use crate::models::rooms_manager::{PairedGame, RoomsManagerTrait};
use crate::repositories::game_repository::GameRepositoryTrait;
use uuid::Uuid;

pub struct PairingGameService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> PairingGameService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    // TODO: Treat disconnection of waiting player
    pub async fn execute(&self, player_id: Uuid) -> Result<Uuid> {
        let paired_game = self.rooms_manager.pair_new_player();

        let player = self.game_repository.get_player(player_id).await?;

        let paired_game_id = match paired_game {
            PairedGame::NewGame(game_id) => {
                self.rooms_manager.create_room(game_id);
                self.rooms_manager
                    .add_player(game_id, player, Some(PlayerColor::White))
                    .unwrap();

                self.game_repository
                    .save_game(GameRecord {
                        id: game_id,
                        white_player: Some(player_id),
                        ..Default::default()
                    })
                    .await?;

                game_id
            }

            PairedGame::ExistingGame(game_id) => {
                self.rooms_manager
                    .add_player(game_id, player, Some(PlayerColor::Black))
                    .unwrap();

                self.game_repository
                    .add_player(game_id, player_id, PlayerColor::Black)
                    .await?;

                game_id
            }
        };

        Ok(paired_game_id)
    }
}
