use crate::http::{error::GenericError, extractor::COOKIE_NAME, Result};
use aide::transform::TransformOperation;
use axum::{
    http::{header::SET_COOKIE, HeaderName},
    response::AppendHeaders,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

fn build_set_cookie(token: Option<String>) -> String {
    let cookie = Cookie::build((COOKIE_NAME, token.unwrap_or_default()))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict);

    cookie.to_string()
}

pub async fn route() -> Result<(AppendHeaders<[(HeaderName, String); 1]>, ())> {
    Ok((AppendHeaders([(SET_COOKIE, build_set_cookie(None))]), ()))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Logout")
        .description("Logout user")
        .response::<200, ()>()
        .response::<400, Json<GenericError>>()
}
