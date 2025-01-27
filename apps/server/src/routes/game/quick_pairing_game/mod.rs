use crate::http::{extractor::AuthUser, Result};
use crate::models::rooms_manager::RoomsManager;
use crate::repositories::game_repository::GameRepository;
use aide::transform::TransformOperation;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use service::PairingGameService;
use uuid::Uuid;

use crate::http::error::GenericError;

mod service;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameId {
    pub game_id: Uuid,
}

fn resource() -> PairingGameService<GameRepository, RoomsManager> {
    PairingGameService::new(GameRepository::new(), RoomsManager::new())
}

pub async fn route(auth_user: AuthUser) -> Result<Json<GameId>> {
    let pairing_service = resource();

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
