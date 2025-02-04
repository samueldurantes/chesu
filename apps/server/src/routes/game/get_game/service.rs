use uuid::Uuid;

use crate::http::{Error, Result};
use crate::models::{Player, RoomsManagerTrait};
use crate::repositories::{GameRepositoryTrait, GameWithPlayers};

pub struct GetGameService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

fn get_mocked_player() -> Player {
    Player {
        id: Uuid::new_v4(),
        username: String::from("Waiting player..."),
        email: String::new(),
    }
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> GetGameService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, room_id: Uuid) -> Result<GameWithPlayers> {
        let room = self.rooms_manager.get_room(room_id);

        match room {
            Ok(room) if !room.is_full() => {
                let (white_player, black_player) = match (room.white_player, room.black_player) {
                    (Some(player_id), None) => Ok((
                        self.game_repository.get_player(player_id).await?,
                        get_mocked_player(),
                    )),
                    (None, Some(player_id)) => Ok((
                        get_mocked_player(),
                        self.game_repository.get_player(player_id).await?,
                    )),
                    _ => Err(Error::InternalServerError),
                }?;

                return Ok(GameWithPlayers {
                    id: room_id,
                    white_player,
                    black_player,
                    ..Default::default()
                });
            }
            _ => self.game_repository.get_game_with_players(room_id).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        models::{MockRoomsManagerTrait, Room},
        repositories::MockGameRepositoryTrait,
    };
    use tokio::sync::broadcast;
    use uuid::uuid;

    use super::*;

    #[tokio::test]
    async fn test_request_game() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();

        mock_rooms_manager.expect_get_room().once().returning(|_| {
            Ok(Room {
                request_key: String::from("w-10-0-0"),
                white_player: Some(uuid!("7e72d61a-c7d0-4260-94ab-7c5a3a41ac72")),
                black_player: None,
                tx: broadcast::channel(100).0,
            })
        });

        mock_game_repository
            .expect_get_player()
            .once()
            .withf(|id| id == &uuid!("7e72d61a-c7d0-4260-94ab-7c5a3a41ac72"))
            .returning(|id| {
                Ok(Player {
                    id,
                    ..Default::default()
                })
            });

        let service = GetGameService::new(mock_game_repository, mock_rooms_manager);

        let result = service.execute(Uuid::new_v4()).await.unwrap();

        assert_eq!(
            result.white_player.id,
            uuid!("7e72d61a-c7d0-4260-94ab-7c5a3a41ac72")
        );

        assert_eq!(
            result.black_player.username,
            String::from("Waiting player...")
        );
    }

    #[tokio::test]
    async fn test_real_game() {
        let mut mock_game_repository = MockGameRepositoryTrait::new();
        let mut mock_rooms_manager = MockRoomsManagerTrait::new();

        mock_rooms_manager.expect_get_room().once().returning(|_| {
            Ok(Room {
                request_key: String::from("w-10-0-0"),
                white_player: Some(uuid!("7e72d61a-c7d0-4260-94ab-7c5a3a41ac72")),
                black_player: Some(uuid!("8734278b-1363-42d1-8c24-c13214d23b0b")),
                tx: broadcast::channel(100).0,
            })
        });

        mock_game_repository
            .expect_get_game_with_players()
            .once()
            .withf(|id| id == &uuid!("55bc0856-6b5a-4e5a-b294-bf82921a996a"))
            .returning(|id| {
                Ok(GameWithPlayers {
                    id,
                    white_player: Player {
                        id: uuid!("7e72d61a-c7d0-4260-94ab-7c5a3a41ac72"),
                        ..Default::default()
                    },
                    black_player: Player {
                        id: uuid!("8734278b-1363-42d1-8c24-c13214d23b0b"),
                        ..Default::default()
                    },
                    bet_value: 10,
                    ..Default::default()
                })
            });

        let service = GetGameService::new(mock_game_repository, mock_rooms_manager);

        let result = service
            .execute(uuid!("55bc0856-6b5a-4e5a-b294-bf82921a996a"))
            .await
            .unwrap();

        assert_eq!(
            result.white_player.id,
            uuid!("7e72d61a-c7d0-4260-94ab-7c5a3a41ac72")
        );

        assert_eq!(
            result.black_player.id,
            uuid!("8734278b-1363-42d1-8c24-c13214d23b0b")
        );

        assert_eq!(result.bet_value, 10);
    }
}
