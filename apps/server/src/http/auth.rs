use crate::http::{error::Error, extractor::AuthUser, Result};
use aide::{
    axum::{routing::get_with, routing::post_with, ApiRouter},
    transform::TransformOperation,
};
use anyhow::Context;
use argon2::{Argon2, PasswordHash};
use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderName},
    response::AppendHeaders,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::{error::GenericError, extractor::COOKIE_NAME};

pub(crate) fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route("/auth/login", post_with(login, login_docs))
        .api_route("/auth/logout", get_with(logout, logout_docs))
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct UserBody<T> {
    user: T,
}

#[derive(Validate, Deserialize, JsonSchema)]
struct LoginUser {
    #[validate(email(message = "Invalid email"))]
    email: String,
    password: String,
}

#[derive(Validate, Deserialize, JsonSchema)]
struct RegisterUser {
    _username: String,
    #[validate(email(message = "Invalid email"))]
    email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct User {
    id: String,
    username: String,
    email: String,
}

fn build_set_cookie(token: Option<String>) -> String {
    let cookie = Cookie::build((COOKIE_NAME, token.unwrap_or_default()))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict);

    cookie.to_string()
}

async fn logout() -> Result<(AppendHeaders<[(HeaderName, String); 1]>, ())> {
    Ok((AppendHeaders([(SET_COOKIE, build_set_cookie(None))]), ()))
}

fn logout_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Logout")
        .description("Logout user")
        .response::<200, ()>()
        .response::<400, Json<GenericError>>()
}

async fn login(
    state: State<crate::AppState>,
    Json(payload): Json<UserBody<LoginUser>>,
) -> Result<(
    AppendHeaders<[(HeaderName, String); 1]>,
    Json<UserBody<User>>,
)> {
    if let Some(message) = validate_user_payload(&payload) {
        return Err(Error::BadRequest { message });
    }

    let user = sqlx::query!(
        r#"
            SELECT id, username, email, password FROM users WHERE email = $1
        "#,
        payload.user.email,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| Error::BadRequest {
        message: "Email and/or password incorrect".to_string(),
    })?;

    verify_password(payload.user.password, user.password).await?;

    let token = AuthUser { user_id: user.id }.to_jwt();

    Ok((
        AppendHeaders([(SET_COOKIE, build_set_cookie(Some(token)))]),
        Json(UserBody {
            user: User {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
            },
        }),
    ))
}

fn login_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Auth")
        .description("Login an user")
        .response::<200, Json<UserBody<User>>>()
        .response::<400, Json<GenericError>>()
}

async fn verify_password(password: String, password_hash: String) -> Result<()> {
    tokio::task::spawn_blocking(move || -> Result<()> {
        let hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

        hash.verify_password(&[&Argon2::default()], password)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => Error::BadRequest {
                    message: "Email and/or password incorrect".to_string(),
                },
                _ => anyhow::anyhow!("Failed to verify password hash: {}", e).into(),
            })
    })
    .await
    .context("Panic in verifying password hash")?
}

fn validate_user_payload<T: Validate>(user_body: &UserBody<T>) -> Option<String> {
    let validation = user_body.user.validate();

    match validation {
        Ok(_) => None,
        Err(errs) => {
            let validation_errs = errs.field_errors();

            if let Some((_, err)) = validation_errs.iter().next() {
                // TODO: Improve this
                let message = err[0].message.clone()?;

                return Some(format!("{}", message));
            }

            None
        }
    }
}
