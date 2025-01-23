use aide::axum::{routing::post_with, ApiRouter};
use sqlx::{Pool, Postgres};

pub mod register;

pub fn router(db: Pool<Postgres>) -> ApiRouter<crate::AppState> {
    ApiRouter::new().api_route(
        "/auth/register",
        post_with(
            |args| register::route(register::service(db), args),
            register::docs,
        ),
    )
}
