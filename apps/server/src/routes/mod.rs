use crate::http;
use aide::axum::ApiRouter;

mod docs;
mod user;
mod wallet;

pub fn mount() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .merge(user::router())
        .merge(wallet::router())
        .merge(docs::router())
        .merge(http::game::router())
}
