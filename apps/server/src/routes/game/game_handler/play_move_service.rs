use crate::http::Result;
use crate::models::{Event, MoveInfo, RoomsManagerTrait};
use crate::repositories::{GameRepositoryTrait, WalletRepositoryTrait};

use super::disconnect_service::resolve_bet;

pub struct PlayMoveService<R: GameRepositoryTrait, M: RoomsManagerTrait, W: WalletRepositoryTrait> {
    game_repository: R,
    rooms_manager: M,
    wallet_repository: W,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait, W: WalletRepositoryTrait>
    PlayMoveService<R, M, W>
{
    pub fn new(game_repository: R, rooms_manager: M, wallet_repository: W) -> Self {
        Self {
            game_repository,
            rooms_manager,
            wallet_repository,
        }
    }

    pub async fn execute(&self, info: MoveInfo) -> Result<(), String> {
        let mut game = self.game_repository.get_game(info.game_id).await?;

        if game.get_player_color(info.player_id)? != game.get_turn_color() {
            return Err(String::from("It's not your turn!"));
        }

        if let Some(new_game_state) = game.check_move(&info.move_played)? {
            game.state = new_game_state;

            self.game_repository
                .update_state(info.game_id, new_game_state)
                .await?;

            resolve_bet(&self.wallet_repository, &game).await?;

            self.rooms_manager
                .get_room(info.game_id)?
                .relay_event(Event::GameChangeState(new_game_state));
        }

        self.game_repository
            .record_move(info.game_id, info.move_played)
            .await?;

        self.rooms_manager
            .handle_move_time(info.game_id, info.player_id)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::Error;
    use crate::models::{Game, GameState, MockRoomsManagerTrait, MoveInfo};
    use crate::repositories::{MockGameRepositoryTrait, MockWalletRepositoryTrait};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_not_player_try_play_move() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mock_rooms_manager = MockRoomsManagerTrait::new();
        let mock_wallet_repository = MockWalletRepositoryTrait::new();

        mock_game_repository.expect_get_game().returning(|_| {
            Ok(Game {
                id: Uuid::new_v4(),
                white_player: Uuid::new_v4(),
                black_player: Uuid::new_v4(),
                state: GameState::Waiting,
                time: 10,
                additional_time: 0,
                bet_value: 0,
                moves: vec![],
            })
        });
        let service = PlayMoveService::new(
            mock_game_repository,
            mock_rooms_manager,
            mock_wallet_repository,
        );

        let input = MoveInfo {
            player_id: Uuid::new_v4(),
            game_id: Uuid::new_v4(),
            move_played: String::from("e4"),
        };

        let result = service.execute(input).await;

        assert!(result.is_err());
        assert_eq!(result, Err(String::from("You are not playing this game!")));
    }

    #[tokio::test]
    async fn test_not_turned_player_try_play_move() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mock_rooms_manager = MockRoomsManagerTrait::new();
        let mock_wallet_repository = MockWalletRepositoryTrait::new();

        mock_game_repository.expect_get_game().returning(|_| {
            Ok(Game {
                id: Uuid::new_v4(),
                white_player: Uuid::new_v4(),
                black_player: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
                ..Default::default()
            })
        });

        let service = PlayMoveService::new(
            mock_game_repository,
            mock_rooms_manager,
            mock_wallet_repository,
        );

        let input = MoveInfo {
            player_id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
            game_id: Uuid::new_v4(),
            move_played: String::from("e4"),
        };

        let result = service.execute(input).await;

        assert!(result.is_err());
        assert_eq!(result, Err(String::from("It's not your turn!")));
    }

    #[tokio::test]
    async fn test_right_player_play_move() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mock_rooms_manager = MockRoomsManagerTrait::new();
        let mock_wallet_repository = MockWalletRepositoryTrait::new();

        mock_game_repository.expect_get_game().returning(|_| {
            Ok(Game {
                id: Uuid::new_v4(),
                white_player: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
                black_player: Uuid::new_v4(),
                ..Default::default()
            })
        });

        mock_game_repository
            .expect_record_move()
            .once()
            .returning(|_, _| Ok(()));

        let service = PlayMoveService::new(
            mock_game_repository,
            mock_rooms_manager,
            mock_wallet_repository,
        );

        let input = MoveInfo {
            player_id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
            game_id: Uuid::new_v4(),
            move_played: String::from("e4"),
        };

        let result = service.execute(input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_game_not_found() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mock_rooms_manager = MockRoomsManagerTrait::new();
        let mock_wallet_repository = MockWalletRepositoryTrait::new();

        mock_game_repository
            .expect_get_game()
            .once()
            .returning(|_| {
                Err(Error::NotFound {
                    message: String::from("Item not found!"),
                })
            });

        let input = MoveInfo {
            player_id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
            game_id: Uuid::new_v4(),
            move_played: String::from("e4"),
        };

        let service = PlayMoveService::new(
            mock_game_repository,
            mock_rooms_manager,
            mock_wallet_repository,
        );

        let result = service.execute(input).await;

        assert!(result.is_err());
        assert_eq!(result, Err(String::from("Item not found!")));
    }
}
