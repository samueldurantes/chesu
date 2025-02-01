use crate::http::{extractor::AuthUser, Error, Result};
use crate::models::game_request::GameRequest;
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

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GameRequestBody {
    key: String,
}

fn resource() -> PairingGameService<GameRepository, RoomsManager> {
    PairingGameService::new(GameRepository::new(), RoomsManager::new())
}

pub async fn route(
    auth_user: AuthUser,
    Json(payload): Json<GameRequestBody>,
) -> Result<Json<GameId>> {
    let pairing_service = resource();

    let game_request = GameRequest::from_str(&payload.key).map_err(|_| Error::BadRequest {
        message: String::from("Invalid game request key"),
    })?;

    let game_id = pairing_service
        .execute(auth_user.user_id, game_request)
        .await?;

    Ok(Json(GameId { game_id }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Quick Pairing")
        .description("Quick Pair players to play")
        .response::<200, Json<GameId>>()
        .response::<400, Json<GenericError>>()
}
