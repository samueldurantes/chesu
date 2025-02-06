use crate::http::{Error, Result};
use crate::internal_error;
use crate::models::{CreateRoomInfo, Game, GameRequest, PairedGame, RoomsManagerTrait};
use crate::repositories::{GameRepositoryTrait, SaveOutgoing, WalletRepositoryTrait};
use uuid::Uuid;

pub struct PairingGameService<
    R: GameRepositoryTrait,
    M: RoomsManagerTrait,
    W: WalletRepositoryTrait,
> {
    game_repository: R,
    rooms_manager: M,
    wallet_repository: W,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait, W: WalletRepositoryTrait>
    PairingGameService<R, M, W>
{
    pub fn new(game_repository: R, rooms_manager: M, wallet_repository: W) -> Self {
        Self {
            game_repository,
            rooms_manager,
            wallet_repository,
        }
    }

    pub async fn execute(&self, player_id: Uuid, game_request: GameRequest) -> Result<Uuid> {
        let balance = self.wallet_repository.get_balance(player_id).await?;

        if balance < game_request.bet_value {
            return Err(Error::BadRequest {
                message: String::from("You don't have enough money! Deposit more sats."),
            });
        }

        let paired_game = self.rooms_manager.pair_new_player(&game_request.key);

        let paired_game_id = match paired_game {
            PairedGame::NewGame(game_id) => {
                self.rooms_manager.create_room(CreateRoomInfo {
                    room_id: game_id,
                    time: game_request.time,
                    additional_time: game_request.additional_time,
                    request_key: game_request.key,
                });
                self.rooms_manager
                    .add_player(game_id, player_id, game_request.player_color)?;

                game_id
            }

            PairedGame::ExistingGame(game_id) => {
                self.rooms_manager.add_player(game_id, player_id, None)?;

                let room = self.rooms_manager.get_room(game_id)?;

                let game = Game {
                    id: game_id,
                    white_player: room.white_player.ok_or(internal_error!())?,
                    black_player: room.black_player.ok_or(internal_error!())?,
                    time: game_request.time,
                    additional_time: game_request.additional_time,
                    bet_value: game_request.bet_value,
                    ..Default::default()
                };

                self.handle_game_save(game).await?;

                game_id
            }
        };

        Ok(paired_game_id)
    }

    async fn handle_game_save(&self, game: Game) -> Result<()> {
        let white_player = game.white_player;
        let black_player = game.black_player;
        let bet_value = game.bet_value;

        self.game_repository.save_game(game).await?;

        self.wallet_repository
            .save_outgoing(SaveOutgoing {
                user_id: white_player,
                amount: bet_value,
            })
            .await
            .map(|_| Error::BadRequest {
                message: String::from("You or your opponent don't have enough money!"),
            })?;

        self.wallet_repository
            .save_outgoing(SaveOutgoing {
                user_id: black_player,
                amount: bet_value,
            })
            .await
            .map(|_| Error::BadRequest {
                message: String::from("You or your opponent don't have enough money!"),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{GameRequest, MockRoomsManagerTrait, PairedGame, Player, PlayerColor};
    use crate::repositories::{MockGameRepositoryTrait, MockWalletRepositoryTrait};
    use mockall::predicate::*;
    use uuid::uuid;

    #[tokio::test]
    async fn quick_pairing_service() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();
        let mut mock_wallet_repository = MockWalletRepositoryTrait::new();

        let request_key = "w-10-0-10";
        let player = Player {
            id: uuid!("5d6cc3e8-8eec-4dab-881f-fddfb831cc41"),
            ..Default::default()
        };

        mock_wallet_repository
            .expect_get_balance()
            .once()
            .returning(|_| Ok(10000));

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

        mock_rooms_manager.expect_create_room().return_const(());

        mock_rooms_manager
            .expect_add_player()
            .returning(|_, _, c| Ok(c.unwrap_or(PlayerColor::White)));

        mock_wallet_repository
            .expect_save_outgoing()
            .withf(|info| info.amount == 10)
            .returning(|_| Ok(Uuid::new_v4()));

        let service = PairingGameService::new(
            mock_game_repository,
            mock_rooms_manager,
            mock_wallet_repository,
        );

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
