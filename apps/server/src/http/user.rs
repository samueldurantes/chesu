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

#[derive(Serialize, Deserialize, JsonSchema)]
struct UserWithBalance {
    id: String,
    username: String,
    email: String,
    balance: i32,
}

impl UserWithBalance {
    pub async fn from_id(db: &Pool<Postgres>, id: Uuid) -> Result<Self, sqlx::Error> {
        let balance = sqlx::query_scalar!(
            r#"
                SELECT COALESCE(last_balance, 0) AS last_balance
                FROM transactions
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT 1
        "#,
            id
        )
        .fetch_optional(db)
        .await?
        .unwrap_or(Some(0))
        .unwrap();

        let User {
            id,
            username,
            email,
        } = sqlx::query_as!(
            User,
            r#" SELECT id, username, email FROM users WHERE id = $1 "#,
            id,
        )
        .fetch_one(db)
        .await?;

        Ok(UserWithBalance {
            id,
            username,
            email,
            balance,
        })
    }
}

async fn me(
    auth_user: AuthUser,
    state: State<crate::AppState>,
) -> Result<Json<UserBody<UserWithBalance>>> {
    tracing::info!("{}", auth_user.user_id);

    Ok(Json(UserBody {
        user: UserWithBalance::from_id(&state.db, auth_user.user_id).await?,
    }))
}

fn me_docs(op: TransformOperation) -> TransformOperation {
    op.tag("User")
        .description("Get logged user")
        .response::<200, Json<UserBody<UserWithBalance>>>()
        .response::<404, Json<GenericError>>()
}
