use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};

mod game_handler;
mod get_game;
mod pairing_game;

pub fn router() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/game/pairing",
            post_with(pairing_game::route, pairing_game::docs),
        )
        .api_route("/game/:id", get_with(get_game::route, get_game::docs))
        .api_route(
            "/game/ws",
            get_with(game_handler::route, game_handler::docs),
        )
}
