use crate::http::Result;
use crate::models::event::Event;
use crate::models::event::MoveInfo;
use crate::models::rooms_manager::RoomsManagerTrait;
use crate::repositories::game_repository::GameRepositoryTrait;

pub struct PlayMoveService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> PlayMoveService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, info: MoveInfo) -> Result<(), String> {
        let game = self.game_repository.get_game(info.game_id).await?;

        if game.get_player_color(info.player_id)? != game.get_turn_color() {
            return Err(String::from("It's not your turn!"));
        }

        if let Some(new_game_state) = game.check_move(&info.move_played)? {
            self.game_repository
                .update_state(info.game_id, new_game_state.clone())
                .await?;

            self.rooms_manager
                .get_room(info.game_id)?
                .relay_event(Event::GameChangeState(new_game_state));
        }

        self.game_repository
            .record_move(info.game_id, info.move_played)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::Error;
    use crate::models::{
        event::MoveInfo,
        game::{Game, GameState},
        rooms_manager::MockRoomsManagerTrait,
    };
    use crate::repositories::game_repository::MockGameRepositoryTrait;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_not_player_try_play_move() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mock_rooms_manager = MockRoomsManagerTrait::new();

        mock_game_repository.expect_get_game().returning(|_| {
            Ok(Game {
                id: Uuid::new_v4(),
                white_player: Uuid::new_v4(),
                black_player: Uuid::new_v4(),
                state: GameState::Waiting,
                bet_value: 0,
                moves: vec![],
            })
        });
        let service = PlayMoveService::new(mock_game_repository, mock_rooms_manager);

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

        mock_game_repository.expect_get_game().returning(|_| {
            Ok(Game {
                id: Uuid::new_v4(),
                white_player: Uuid::new_v4(),
                black_player: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
                ..Default::default()
            })
        });

        let service = PlayMoveService::new(mock_game_repository, mock_rooms_manager);

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

        let service = PlayMoveService::new(mock_game_repository, mock_rooms_manager);

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

        let service = PlayMoveService::new(mock_game_repository, mock_rooms_manager);

        let result = service.execute(input).await;

        assert!(result.is_err());
        assert_eq!(result, Err(String::from("Item not found!")));
    }
}
