use super::game::GameState;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum Event {
    PlayMove(MoveInfo),
    Disconnect(DisconnectInfo),
    GameChangeState(GameState),
    Join,
}

impl Event {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<Event>(&json)
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize event!")
    }
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisconnectInfo {
    pub game_id: Uuid,
    pub player_id: Uuid,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoveInfo {
    pub game_id: Uuid,
    pub player_id: Uuid,
    pub move_played: String,
}
