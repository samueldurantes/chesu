use crate::{
    http::Result, models::game::Game, repositories::game_repository::GameRepository,
    repositories::game_repository::GameRepositoryTrait,
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

fn resource() -> GameRepository {
    GameRepository::new()
}

pub async fn route(Path(GameId { id: game_id }): Path<GameId>) -> Result<Json<GameBody>> {
    let game_repository = resource();

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
