use crate::http::{Error, Result};
use crate::internal_error;
use crate::models::{Game, GameRequest, PairedGame, RoomsManagerTrait};
use crate::repositories::GameRepositoryTrait;
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

    pub async fn execute(&self, player_id: Uuid, game_request: GameRequest) -> Result<Uuid> {
        let paired_game = self.rooms_manager.pair_new_player(&game_request.key);

        let paired_game_id = match paired_game {
            PairedGame::NewGame(game_id) => {
                self.rooms_manager.create_room(game_id, &game_request.key);
                self.rooms_manager
                    .add_player(game_id, player_id, game_request.player_color)?;

                game_id
            }

            PairedGame::ExistingGame(game_id) => {
                self.rooms_manager.add_player(game_id, player_id, None)?;

                let room = self.rooms_manager.get_room(game_id).unwrap();

                let game = Game {
                    id: game_id,
                    white_player: room.white_player.ok_or(internal_error!())?,
                    black_player: room.black_player.ok_or(internal_error!())?,
                    bet_value: game_request.bet_value,
                    ..Default::default()
                };

                self.game_repository.save_game(game).await?;

                game_id
            }
        };

        Ok(paired_game_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{GameRequest, MockRoomsManagerTrait, PairedGame, Player, PlayerColor};
    use crate::repositories::MockGameRepositoryTrait;
    use mockall::predicate::*;
    use uuid::uuid;

    #[tokio::test]
    async fn quick_pairing_service() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();

        let request_key = "w-10-0-0";
        let player = Player {
            id: uuid!("5d6cc3e8-8eec-4dab-881f-fddfb831cc41"),
            ..Default::default()
        };

        mock_rooms_manager
            .expect_pair_new_player()
            .returning(|_| PairedGame::NewGame(uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a")));

        mock_game_repository
            .expect_get_player()
            .with(eq(player.id))
            .returning(|p_id| {
                Ok(Player {
                    id: p_id,
                    ..Default::default()
                })
            });

        mock_rooms_manager
            .expect_create_room()
            .with(
                eq(uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a")),
                eq(request_key.to_string()),
            )
            .return_const(());

        mock_rooms_manager
            .expect_add_player()
            .returning(|_, _, c| Ok(c.unwrap_or(PlayerColor::White)));

        let service = PairingGameService::new(mock_game_repository, mock_rooms_manager);

        let game_request = GameRequest::from_str(request_key);

        assert!(game_request.is_ok());

        let result = service.execute(player.id, game_request.unwrap()).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a")
        );
    }
}
