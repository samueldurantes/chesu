use crate::{
    http::Result,
    models::{
        game::Player,
        rooms_manager::{RoomsManager, RoomsManagerTrait},
    },
    repositories::game_repository::{GameRepository, GameRepositoryTrait, GameWithPlayers},
    Error,
};
use aide::transform::TransformOperation;
use anyhow::anyhow;
use axum::{extract::Path, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::error::GenericError;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameId {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, JsonSchema, Default)]
pub struct GameBody {
    pub game: GameWithPlayers,
}

fn resource() -> (GameRepository, RoomsManager) {
    (GameRepository::new(), RoomsManager::new())
}

fn get_mocked_player() -> Player {
    Player {
        id: Uuid::new_v4(),
        username: String::from("Waiting player..."),
        email: String::new(),
    }
}

pub async fn route(Path(GameId { id: game_id }): Path<GameId>) -> Result<Json<GameBody>> {
    let (game_repository, rooms_manager) = resource();

    let room = rooms_manager.get_room(game_id);

    if let Some(room) = room {
        if !room.is_full() {
            let (white_player, black_player) = match (room.white_player, room.black_player) {
                (Some(player_id), None) => Ok((
                    game_repository.get_player(player_id).await?,
                    get_mocked_player(),
                )),
                (None, Some(player_id)) => Ok((
                    get_mocked_player(),
                    game_repository.get_player(player_id).await?,
                )),
                _ => Err(Error::Anyhow(anyhow!(""))),
            }?;

            return Ok(Json(GameBody {
                game: GameWithPlayers {
                    id: game_id,
                    white_player,
                    black_player,
                    ..Default::default()
                },
            }));
        }
    }

    Ok(Json(GameBody {
        game: game_repository.get_game_with_players(game_id).await?,
    }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Get a game")
        .response::<200, Json<GameBody>>()
        .response::<400, Json<GenericError>>()
        .response::<404, Json<GenericError>>()
}
