use crate::repositories::game_repository::GameRepositoryTrait;
use crate::{http::Result, models::rooms_manager::RoomsManagerTrait};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisconnectInfo {
    pub game_id: Uuid,
    pub player_id: Uuid,
}

pub struct DisconnectService<R: GameRepositoryTrait, M: RoomsManagerTrait> {
    game_repository: R,
    rooms_manager: M,
}

impl<R: GameRepositoryTrait, M: RoomsManagerTrait> DisconnectService<R, M> {
    pub fn new(game_repository: R, rooms_manager: M) -> Self {
        Self {
            game_repository,
            rooms_manager,
        }
    }

    pub async fn execute(&self, info: DisconnectInfo) -> Result<(), String> {
        Ok(())
    }
}
