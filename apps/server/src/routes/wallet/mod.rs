use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};

mod check_invoice;
mod create_invoice;
mod deposit_webhook;
mod withdraw;

pub fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route(
            "/invoice/create",
            post_with(create_invoice::route, create_invoice::docs),
        )
        .api_route(
            "/invoice/check",
            get_with(check_invoice::route, check_invoice::docs),
        )
        .api_route(
            "/invoice/withdraw",
            post_with(withdraw::route, withdraw::docs),
        )
        .api_route(
            "/invoice/settled",
            post_with(deposit_webhook::route, deposit_webhook::docs),
        )
}
