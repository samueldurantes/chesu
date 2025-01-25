use crate::http::{extractor::AuthUser, Result};
use crate::models::rooms_manager::RoomsManager;
use crate::repositories::game_repository::GameRepository;
use crate::services::game::create_game_service::CreateGameService;
use aide::transform::TransformOperation;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::error::GenericError;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameId {
    pub game_id: Uuid,
}

pub fn resource() -> CreateGameService<GameRepository, RoomsManager> {
    CreateGameService::new(GameRepository::new(), RoomsManager::new())
}

pub async fn route(
    create_game_service: CreateGameService<GameRepository, RoomsManager>,
    auth_user: AuthUser,
) -> Result<Json<GameId>> {
    let game = create_game_service.execute(auth_user.user_id).await?;

    Ok(Json(GameId { game_id: game }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Create a game")
        .response::<200, Json<GameId>>()
        .response::<400, Json<GenericError>>()
}
