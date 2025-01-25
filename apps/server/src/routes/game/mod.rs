use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};

mod create_game;
mod game_handler;
mod get_game;
mod join_game;
mod quick_pairing_game;

pub fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route(
            "/game/pairing",
            post_with(
                |auth_user| quick_pairing_game::route(quick_pairing_game::resource(), auth_user),
                quick_pairing_game::docs,
            ),
        )
        .api_route(
            "/game/create",
            post_with(
                |auth_user| create_game::route(create_game::resource(), auth_user),
                create_game::docs,
            ),
        )
        .api_route(
            "/game/:id",
            post_with(
                |auth_user, path| join_game::route(join_game::resource(), auth_user, path),
                join_game::docs,
            ),
        )
        .api_route(
            "/game/:id",
            get_with(
                |path| get_game::route(get_game::resource(), path),
                get_game::docs,
            ),
        )
        .api_route(
            "/game/ws",
            get_with(
                |ws| game_handler::route(game_handler::resource(), ws),
                game_handler::docs,
            ),
        )
}
