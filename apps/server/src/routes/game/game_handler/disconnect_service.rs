use uuid::Uuid;

use crate::models::{
    event::{DisconnectInfo, Event},
    game::{Game, GameState},
};
use crate::repositories::game_repository::GameRepositoryTrait;
use crate::{http::Result, models::rooms_manager::RoomsManagerTrait};

pub struct DisconnectService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
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

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> DisconnectService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, info: DisconnectInfo) -> Result<(), String> {
        let room = self.rooms_manager.get_room(info.game_id)?;

        if !room.is_playing(info.player_id) {
            return Ok(());
        }

        self.rooms_manager.remove_room(info.game_id);

        if !room.is_full() {
            return Ok(self.rooms_manager.remove_request(&room.request_key));
        }

        let game = self.game_repository.get_game(info.game_id).await?;

        if let Some(new_game_state) = check_new_game_state(&game, info.player_id) {
            self.game_repository
                .update_state(game.id, new_game_state)
                .await?;

            room.relay_event(Event::GameChangeState(new_game_state));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        event::DisconnectInfo,
        game::{Game, GameState},
        rooms_manager::{MockRoomsManagerTrait, Room},
    };
    use crate::repositories::game_repository::MockGameRepositoryTrait;
    use tokio::sync::broadcast;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_player_disconnection_from_request() {
        let mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();

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

        let input = DisconnectInfo {
            game_id: uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"),
            player_id: uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"),
        };

        let service = DisconnectService::new(mock_game_repository, mock_rooms_manager);

        let result = service.execute(input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_player_disconnection_from_game() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();

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

        let service = DisconnectService::new(mock_game_repository, mock_rooms_manager);

        let result = service.execute(input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_viewer_disconnection() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();

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

        let service = DisconnectService::new(mock_game_repository, mock_rooms_manager);

        let result = service.execute(input).await;

        assert!(result.is_ok());
    }
}
