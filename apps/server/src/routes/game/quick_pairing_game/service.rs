use crate::http::{Error, Result};
use crate::internal_error;
use crate::models::game::{Game, PlayerColor};
use crate::models::rooms_manager::{PairedGame, RoomsManagerTrait};
use crate::repositories::game_repository::GameRepositoryTrait;
use uuid::Uuid;

pub struct PairingGameService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

#[derive(Debug, PartialEq)]
pub struct GameRequest {
    pub key: String,
    pub player_color: Option<PlayerColor>,
    pub _total_time: u8,
    pub _turn_time: u8,
    pub bet_value: i32,
}

impl GameRequest {
    pub fn from_str(key: &str) -> Result<Self, ()> {
        let mut result = key.split("-");
        let player_color = result
            .next()
            .map(|c| match c {
                "w" => Ok(Some(PlayerColor::White)),
                "b" => Ok(Some(PlayerColor::Black)),
                "n" => Ok(None),
                _ => Err(()),
            })
            .unwrap_or(Err(()))?;
        let _total_time = result
            .next()
            .map(|r| r.parse::<u8>().map_err(|_| ()))
            .unwrap_or(Err(()))?;
        let _turn_time = result
            .next()
            .map(|r| r.parse::<u8>().map_err(|_| ()))
            .unwrap_or(Err(()))?;
        let bet_value = result
            .next()
            .map(|r| r.parse::<i32>().map_err(|_| ()))
            .unwrap_or(Err(()))?;

        if _total_time <= 0 || bet_value < 0 {
            return Err(());
        }

        Ok(Self {
            key: key.to_string(),
            _total_time,
            _turn_time,
            bet_value,
            player_color,
        })
    }
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> PairingGameService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, player_id: Uuid, game_request: GameRequest) -> Result<Uuid> {
        let paired_game = self.rooms_manager.pair_new_player(game_request.key.clone());

        let paired_game_id = match paired_game {
            PairedGame::NewGame(game_id) => {
                self.rooms_manager.create_room(game_id, game_request.key);
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
    use crate::models::rooms_manager::PairedGame;
    use crate::{
        models::{game::Player, game::PlayerColor, rooms_manager::MockRoomsManagerTrait},
        repositories::game_repository::MockGameRepositoryTrait,
    };
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

    #[test]
    fn test_request_key_parsing_1() {
        let input = "";
        let result = GameRequest::from_str(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_request_key_parsing_2() {
        let input = "w-10-0-0";
        let result = GameRequest::from_str(input);

        assert!(result.is_ok());
        assert_eq!(
            result,
            Ok(GameRequest {
                key: input.to_string(),
                player_color: Some(PlayerColor::White),
                _total_time: 10,
                _turn_time: 0,
                bet_value: 0,
            })
        )
    }

    #[test]
    fn test_request_key_parsing_3() {
        let input = "j-10-0-0";
        let result = GameRequest::from_str(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_request_key_parsing_4() {
        let input = "b-30-10-10000";
        let result = GameRequest::from_str(input);

        assert!(result.is_ok());
        assert_eq!(
            result,
            Ok(GameRequest {
                key: input.to_string(),
                player_color: Some(PlayerColor::Black),
                _total_time: 30,
                _turn_time: 10,
                bet_value: 10000,
            })
        )
    }
}
