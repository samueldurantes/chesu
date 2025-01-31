use crate::{
    http::Result,
    models::{
        game::Game,
        rooms_manager::{RoomsManager, RoomsManagerTrait},
    },
    repositories::game_repository::{GameRepository, GameRepositoryTrait},
};
use aide::transform::TransformOperation;
use axum::{extract::Path, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::error::GenericError;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameId {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameBody {
    pub game: Game,
}

fn resource() -> (GameRepository, RoomsManager) {
    (GameRepository::new(), RoomsManager::new())
}

pub async fn route(Path(GameId { id: game_id }): Path<GameId>) -> Result<Json<GameBody>> {
    let (game_repository, rooms_manager) = resource();

    let room = rooms_manager.get_room(game_id);

    if let Some(room) = room {
        if !room.is_full() {
            return Ok(Json(GameBody {
                game: Game {
                    id: game_id,
                    white_player: room.white_player,
                    black_player: room.black_player,
                    ..Default::default()
                },
            }));
        }
    }

    Ok(Json(GameBody {
        game: game_repository.get_game(game_id).await?,
    }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Get a game")
        .response::<200, Json<GameBody>>()
        .response::<400, Json<GenericError>>()
        .response::<404, Json<GenericError>>()
}
