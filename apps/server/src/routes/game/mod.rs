use aide::axum::{routing::post_with, ApiRouter};

mod quick_pairing_game;

pub fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new().api_route(
        "/game/pairing",
        post_with(
            |auth_user| quick_pairing_game::route(quick_pairing_game::resource(), auth_user),
            quick_pairing_game::docs,
        ),
    )
}
