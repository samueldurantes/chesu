use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};

mod login;
mod logout;
mod me;
mod register;

pub fn router() -> ApiRouter {
    ApiRouter::new()
        .api_route("/auth/register", post_with(register::route, register::docs))
        .api_route("/auth/login", post_with(login::route, login::docs))
        .api_route("/auth/logout", get_with(logout::route, logout::docs))
        .api_route("/user/me", get_with(me::route, me::docs))
}
