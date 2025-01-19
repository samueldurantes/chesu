use crate::{
    http::{extractor::AuthUser, Result},
    AppState,
};
use aide::{
    axum::{routing::get_with, ApiRouter},
    transform::TransformOperation,
};
use axum::{extract::State, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
struct User {
    id: String,
    username: String,
    email: String,
}

async fn get_user_by_id(state: &State<AppState>, id: Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#" SELECT id, username, email FROM users WHERE id = $1 "#,
        id,
    )
    .fetch_one(&state.db)
    .await
}

async fn me(auth_user: AuthUser, state: State<crate::AppState>) -> Result<Json<UserBody<User>>> {
    let user = get_user_by_id(&state, auth_user.user_id).await?;

    Ok(Json(UserBody { user }))
}

fn me_docs(op: TransformOperation) -> TransformOperation {
    op.tag("User")
        .description("Get logged user")
        .response::<200, Json<UserBody<User>>>()
        .response::<404, Json<GenericError>>()
}
