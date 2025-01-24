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
            post_with(
                |auth_user, payload| {
                    create_invoice::route(create_invoice::resource(), auth_user, payload)
                },
                create_invoice::docs,
            ),
        )
        .api_route(
            "/invoice/check",
            get_with(
                |auth_user| check_invoice::route(check_invoice::resource(), auth_user),
                check_invoice::docs,
            ),
        )
        .api_route(
            "/invoice/withdraw",
            post_with(
                |auth_user, payload| withdraw::route(withdraw::resource(), auth_user, payload),
                withdraw::docs,
            ),
        )
        .api_route(
            "/invoice/settled",
            post_with(
                |payload| deposit_webhook::route(deposit_webhook::resource(), payload),
                deposit_webhook::docs,
            ),
        )
}
