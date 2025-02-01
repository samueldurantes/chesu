use crate::http::{Error, Result};
use crate::models::game::{Game, PlayerColor};
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
                    .add_player(game_id, player_id, game_request.player_color)
                    .map_err(status_500!())?;

                game_id
            }

            PairedGame::ExistingGame(game_id) => {
                self.rooms_manager
                    .add_player(game_id, player_id, None)
                    .map_err(status_500!())?;

                let room = self.rooms_manager.get_room(game_id).unwrap();

                let game = Game {
                    id: game_id,
                    white_player: room.white_player.ok_or(Error::Anyhow(anyhow!("")))?,
                    black_player: room.black_player.ok_or(Error::Anyhow(anyhow!("")))?,
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
