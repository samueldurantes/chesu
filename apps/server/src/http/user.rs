use crate::http::{extractor::AuthUser, Result};
use aide::{
    axum::{routing::get_with, ApiRouter},
    transform::TransformOperation,
};
use axum::{extract::State, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;
use uuid::Uuid;

use super::error::GenericError;

pub(crate) fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new().api_route("/user/me", get_with(me, me_docs))
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct UserBody<T> {
    user: T,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}

impl User {
    pub async fn from_id(db: &Pool<Postgres>, id: Uuid) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#" SELECT id, username, email FROM users WHERE id = $1 "#,
            id,
        )
        .fetch_one(db)
        .await
    }
}

async fn me(auth_user: AuthUser, state: State<crate::AppState>) -> Result<Json<UserBody<User>>> {
    Ok(Json(UserBody {
        user: User::from_id(&state.db, auth_user.user_id).await?,
    }))
}

fn me_docs(op: TransformOperation) -> TransformOperation {
    op.tag("User")
        .description("Get logged user")
        .response::<200, Json<UserBody<User>>>()
        .response::<404, Json<GenericError>>()
}
