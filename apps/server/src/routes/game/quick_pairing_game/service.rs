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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{game::Player, rooms_manager::MockRoomsManagerTrait},
        repositories::game_repository::MockGameRepositoryTrait,
    };
    use mockall::predicate::*;

    #[tokio::test]
    async fn quick_pairing_service() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();

        let player_id = Uuid::new_v4();
        let game_id = Uuid::new_v4();
        let player = Player {
            id: player_id,
            username: String::from("romero"),
            email: String::from("romero@dias.com"),
        };
        let player_copy = player.clone();

        mock_game_repository
            .expect_get_player()
            .with(eq(player_id))
            .returning(move |_| Ok(player_copy.clone()));

        mock_game_repository
            .expect_save_game()
            .withf(move |game_record| {
                game_record.id == game_id && game_record.white_player == Some(player_id)
            })
            .returning(|_| Ok(()));

        mock_rooms_manager
            .expect_pair_new_player()
            .returning(move || PairedGame::NewGame(game_id));

        mock_rooms_manager
            .expect_create_room()
            .with(eq(game_id))
            .return_const(());

        mock_rooms_manager
            .expect_add_player()
            .with(
                eq(game_id),
                eq(player.clone()),
                eq(Some(PlayerColor::White)),
            )
            .returning(|_, _, _| Ok(PlayerColor::White));

        let service = PairingGameService::new(mock_game_repository, mock_rooms_manager);

        let result = service.execute(player_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), game_id);
    }
}
