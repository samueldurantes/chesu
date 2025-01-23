use crate::{
    http::{extractor::AuthUser, Result},
    repositories::user_repository::UserRepository,
    services::user::me_service::MeService,
};
use aide::transform::TransformOperation;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::error::GenericError;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UserBody<T> {
    user: T,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UserWithoutPassword {
    id: Uuid,
    username: String,
    email: String,
    balance: i32,
}

pub fn service() -> MeService<UserRepository> {
    MeService::new(UserRepository::new())
}

pub async fn route(
    me_service: MeService<UserRepository>,
    auth_user: AuthUser,
) -> Result<Json<UserBody<UserWithoutPassword>>> {
    let crate::models::user::User {
        id,
        email,
        username,
        balance,
        ..
    } = me_service.execute(auth_user.user_id).await?;

    Ok(Json(UserBody {
        user: UserWithoutPassword {
            id,
            email,
            username,
            balance,
        },
    }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("User")
        .description("Get logged user")
        .response::<200, Json<UserBody<UserWithoutPassword>>>()
        .response::<404, Json<GenericError>>()
}
