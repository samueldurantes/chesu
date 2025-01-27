use crate::{
    http::{extractor::AuthUser, Result},
    repositories::user_repository::{UserRepository, UserRepositoryTrait},
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

fn resource() -> UserRepository {
    UserRepository::new()
}

pub async fn route(auth_user: AuthUser) -> Result<Json<UserBody<UserWithoutPassword>>> {
    let user_repository = resource();

    let crate::models::user::User {
        id,
        email,
        username,
        balance,
        ..
    } = user_repository.find_by_id(auth_user.user_id).await?;

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
