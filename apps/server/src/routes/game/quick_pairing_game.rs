use crate::http::{extractor::AuthUser, Result};
use crate::models::rooms_manager::RoomsManager;
use crate::repositories::game_repository::GameRepository;
use crate::services::game::pairing_service::PairingGameService;
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

pub fn resource() -> PairingGameService<GameRepository, RoomsManager> {
    PairingGameService::new(GameRepository::new(), RoomsManager::new())
}

pub async fn route(
    pairing_service: PairingGameService<GameRepository, RoomsManager>,
    auth_user: AuthUser,
) -> Result<Json<GameId>> {
    let paired_game = pairing_service.execute(auth_user.user_id).await?;

    Ok(Json(GameId {
        game_id: paired_game,
    }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Quick Pairing")
        .description("Quick Pair players to play")
        .response::<200, Json<GameId>>()
        .response::<400, Json<GenericError>>()
}
