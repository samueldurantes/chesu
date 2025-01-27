use crate::http::{extractor::AuthUser, Result};
use crate::models::rooms_manager::RoomsManager;
use crate::repositories::game_repository::GameRepository;
use aide::transform::TransformOperation;
use axum::extract::Path;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use service::JoinGameService;
use uuid::Uuid;

use crate::http::error::GenericError;

mod service;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameId {
    pub game_id: Uuid,
}

fn resource() -> JoinGameService<GameRepository, RoomsManager> {
    JoinGameService::new(GameRepository::new(), RoomsManager::new())
}

pub async fn route(
    auth_user: AuthUser,
    Path(GameId { game_id }): Path<GameId>,
) -> Result<Json<GameId>> {
    let join_game_service = resource();

    let game_id = join_game_service
        .execute(auth_user.user_id, game_id)
        .await?;

    Ok(Json(GameId { game_id }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Join a game")
        .response::<200, Json<GameId>>()
        .response::<400, Json<GenericError>>()
        .response::<404, Json<GenericError>>()
}
