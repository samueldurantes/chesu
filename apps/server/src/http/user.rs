use crate::http::{error::Error, extractor::AuthUser, Result};
use aide::{
    axum::{routing::get_with, ApiRouter},
    transform::TransformOperation,
};
use axum::{extract::State, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub(crate) fn router() -> ApiRouter<crate::State> {
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

async fn me(auth_user: AuthUser, state: State<crate::State>) -> Result<Json<UserBody<User>>> {
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT id, username, email
            FROM users
            WHERE id = $1
        "#,
        auth_user.user_id,
    )
    .fetch_optional(&state.db)
    .await?;

    match user {
        Some(user) => Ok(Json(UserBody { user })),
        None => Err(Error::NotFound {
            error: "User not found".to_string(),
        }),
    }
}

fn me_docs(op: TransformOperation) -> TransformOperation {
    op.tag("User")
        .description("Get logged user")
        .response::<200, Json<UserBody<User>>>()
}
