use crate::{
    http::Result,
    models::rooms_manager::RoomsManager,
    repositories::game_repository::{GameRepository, GameWithPlayers},
};
use aide::transform::TransformOperation;
use axum::{extract::Path, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use service::GetGameService;
use uuid::Uuid;

use crate::http::error::GenericError;

mod service;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameId {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, JsonSchema, Default)]
pub struct GameBody {
    pub game: GameWithPlayers,
}

fn resource() -> GetGameService<GameRepository, RoomsManager> {
    GetGameService::new(GameRepository::new(), RoomsManager::new())
}

pub async fn route(Path(GameId { id: game_id }): Path<GameId>) -> Result<Json<GameBody>> {
    let get_game_service = resource();

    Ok(Json(GameBody {
        game: get_game_service.execute(game_id).await?,
    }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Get a game")
        .response::<200, Json<GameBody>>()
        .response::<400, Json<GenericError>>()
        .response::<404, Json<GenericError>>()
}
