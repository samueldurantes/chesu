use crate::http::{Error, Result};
use crate::models::game::{GameRecord, PlayerColor};
use crate::models::rooms_manager::{PairedGame, RoomsManagerTrait};
use crate::repositories::game_repository::GameRepositoryTrait;
use crate::status_500;
use anyhow::anyhow;
use uuid::Uuid;

pub struct PairingGameService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

#[derive(Debug, PartialEq)]
pub struct GameRequest {
    key: String,
    player_color: Option<PlayerColor>,
    _total_time: u8,
    _turn_time: u8,
    bet_value: i32,
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

        let player = self.game_repository.get_player(player_id).await?;

        let paired_game_id = match paired_game {
            PairedGame::NewGame(game_id) => {
                self.rooms_manager.create_room(game_id, game_request.key);
                self.rooms_manager
                    .add_player(game_id, player, game_request.player_color)
                    .map_err(status_500!())?;

                game_id
            }

            PairedGame::ExistingGame(game_id) => {
                self.rooms_manager
                    .add_player(game_id, player, None)
                    .map_err(status_500!())?;

                let room = self.rooms_manager.get_room(game_id).unwrap();

                let game = GameRecord {
                    id: game_id,
                    white_player: room.white_player.map(|p| p.id),
                    black_player: room.black_player.map(|p| p.id),
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

mod tests {
    use super::*;

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
