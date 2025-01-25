use aide::axum::ApiRouter;

mod docs;
mod game;
mod user;
mod wallet;

pub fn mount() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .merge(user::router())
        .merge(wallet::router())
        .merge(docs::router())
        .merge(game::router())
}
