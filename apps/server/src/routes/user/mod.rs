use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};

pub mod login;
pub mod logout;
pub mod me;
pub mod register;

pub fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route(
            "/auth/register",
            post_with(
                |args| register::route(register::service(), args),
                register::docs,
            ),
        )
        .api_route(
            "/auth/login",
            post_with(|args| login::route(login::service(), args), login::docs),
        )
        .api_route("/auth/logout", get_with(logout::route, logout::docs))
        .api_route(
            "/user/me",
            get_with(|args| me::route(me::service(), args), me::docs),
        )
}
