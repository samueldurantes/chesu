use crate::http;
use aide::axum::ApiRouter;
use sqlx::{Pool, Postgres};

pub mod user;

pub fn router(db: Pool<Postgres>) -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .merge(user::router(db))
        .merge(http::wallet::router())
        .merge(http::auth::router())
        .merge(http::docs::router())
        .merge(http::game::router())
        .merge(http::user::router())
}
