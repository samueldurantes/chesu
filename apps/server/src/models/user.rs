use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub balance: i32,
}
