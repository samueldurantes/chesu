use crate::http;
use aide::axum::ApiRouter;

mod docs;
mod user;

pub fn mount() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .merge(user::router())
        .merge(http::wallet::router())
        .merge(docs::router())
        .merge(http::game::router())
        .merge(http::user::router())
}
