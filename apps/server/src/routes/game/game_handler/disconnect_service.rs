use uuid::Uuid;

use crate::http::Result;
use crate::models::{DisconnectInfo, Event, Game, GameRequest, GameState, RoomsManagerTrait};
use crate::repositories::{GameRepositoryTrait, SaveIncoming, WalletRepositoryTrait};

pub struct DisconnectService<R: GameRepositoryTrait, M: RoomsManagerTrait, W: WalletRepositoryTrait>
{
    game_repository: R,
    rooms_manager: M,
    wallet_repository: W,
}

fn check_new_game_state(game: &Game, player_disconnect: Uuid) -> Option<GameState> {
    match (game.state, player_disconnect) {
        (GameState::Waiting, _) => Some(GameState::Draw),
        (GameState::Running, player_id) if player_id == game.white_player => {
            Some(GameState::BlackWin)
        }
        (GameState::Running, player_id) if player_id == game.black_player => {
            Some(GameState::WhiteWin)
        }
        _ => None,
    }
}

pub async fn resolve_bet<W: WalletRepositoryTrait>(
    wallet_repository: &W,
    game: &Game,
) -> Result<()> {
    let (white_amount, black_amount) = match game.state {
        GameState::Draw => (game.bet_value, game.bet_value),
        GameState::WhiteWin => (2 * game.bet_value, 0),
        GameState::BlackWin => (0, 2 * game.bet_value),
        _ => (0, 0),
    };

    if white_amount > 0 {
        wallet_repository
            .save_incoming(SaveIncoming {
                user_id: game.white_player,
                amount: white_amount,
                invoice: None,
            })
            .await?;
    }

    if black_amount > 0 {
        wallet_repository
            .save_incoming(SaveIncoming {
                user_id: game.black_player,
                amount: black_amount,
                invoice: None,
            })
            .await?;
    }

    Ok(())
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait, W: WalletRepositoryTrait>
    DisconnectService<R, M, W>
{
    pub fn new(game_repository: R, rooms_manager: M, wallet_repository: W) -> Self {
        Self {
            game_repository,
            rooms_manager,
            wallet_repository,
        }
    }

    pub async fn execute(&self, info: DisconnectInfo) -> Result<(), String> {
        let room = self.rooms_manager.get_room(info.game_id)?;

        if !room.is_playing(info.player_id) {
            return Ok(());
        }

        self.rooms_manager.remove_room(info.game_id);

        if !room.is_full() {
            self.wallet_repository
                .save_incoming(SaveIncoming {
                    user_id: info.player_id,
                    amount: GameRequest::from_str(&room.request_key)?.bet_value,
                    invoice: None,
                })
                .await?;

            return Ok(self.rooms_manager.remove_request(&room.request_key));
        }

        let mut game = self.game_repository.get_game(info.game_id).await?;

        if let Some(new_game_state) = check_new_game_state(&game, info.player_id) {
            game.state = new_game_state;

            self.game_repository
                .update_state(game.id, new_game_state)
                .await?;

            resolve_bet(&self.wallet_repository, &game).await?;

            room.relay_event(Event::GameChangeState(new_game_state));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DisconnectInfo, Game, GameState, MockRoomsManagerTrait, Room};
    use crate::repositories::{MockGameRepositoryTrait, MockWalletRepositoryTrait};
    use tokio::sync::broadcast;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_player_disconnection_from_request() {
        let mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();
        let mut mock_wallet_repository = MockWalletRepositoryTrait::new();

        mock_rooms_manager.expect_get_room().once().returning(|_| {
            Ok(Room {
                request_key: String::from("w-10-0-0"),
                white_player: Some(uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b")),
                black_player: None,
                tx: broadcast::channel(100).0,
            })
        });

        mock_rooms_manager
            .expect_remove_request()
            .once()
            .withf(|request_key| request_key == "w-10-0-0")
            .returning(|_| ());

        mock_rooms_manager
            .expect_remove_room()
            .once()
            .withf(|id| id == &uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"))
            .returning(|_| ());

        mock_wallet_repository
            .expect_save_incoming()
            .once()
            .returning(|_| Ok(Uuid::new_v4()));

        let input = DisconnectInfo {
            game_id: uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"),
            player_id: uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"),
        };

        let service = DisconnectService::new(
            mock_game_repository,
            mock_rooms_manager,
            mock_wallet_repository,
        );

        let result = service.execute(input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_player_disconnection_from_game() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();
        let mock_wallet_repository = MockWalletRepositoryTrait::new();

        let input = DisconnectInfo {
            game_id: uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"),
            player_id: uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"),
        };

        mock_rooms_manager
            .expect_get_room()
            .once()
            .withf(|id| id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"))
            .returning(|_| {
                Ok(Room {
                    request_key: String::from("w-10-0-0"),
                    white_player: Some(uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade")),
                    black_player: Some(Uuid::new_v4()),
                    tx: broadcast::channel(100).0,
                })
            });

        mock_game_repository
            .expect_get_game()
            .once()
            .withf(|id| id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"))
            .returning(|id| {
                Ok(Game {
                    id,
                    white_player: uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"),
                    black_player: Uuid::new_v4(),
                    bet_value: 0,
                    state: GameState::Running,
                    moves: Vec::new(),
                })
            });

        mock_game_repository
            .expect_update_state()
            .withf(|id, g| {
                id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b")
                    && g == &GameState::BlackWin
            })
            .returning(|_, _| Ok(()));

        mock_rooms_manager
            .expect_remove_room()
            .once()
            .withf(|id| id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"))
            .returning(|_| ());

        let service = DisconnectService::new(
            mock_game_repository,
            mock_rooms_manager,
            mock_wallet_repository,
        );

        let result = service.execute(input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_viewer_disconnection() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();
        let mock_wallet_repository = MockWalletRepositoryTrait::new();

        let input = DisconnectInfo {
            game_id: uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"),
            player_id: Uuid::new_v4(),
        };

        mock_rooms_manager
            .expect_get_room()
            .once()
            .withf(|id| id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"))
            .returning(|_| {
                Ok(Room {
                    request_key: String::from("w-10-0-0"),
                    white_player: Some(Uuid::new_v4()),
                    black_player: Some(Uuid::new_v4()),
                    tx: broadcast::channel(100).0,
                })
            });

        mock_game_repository.expect_get_game().never();
        mock_game_repository.expect_update_state().never();
        mock_rooms_manager.expect_remove_room().never();

        let service = DisconnectService::new(
            mock_game_repository,
            mock_rooms_manager,
            mock_wallet_repository,
        );

        let result = service.execute(input).await;

        assert!(result.is_ok());
    }
}
