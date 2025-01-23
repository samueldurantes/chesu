use crate::http;
use aide::axum::ApiRouter;

pub mod user;

pub fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .merge(user::router())
        .merge(http::wallet::router())
        .merge(http::docs::router())
        .merge(http::game::router())
        .merge(http::user::router())
}
