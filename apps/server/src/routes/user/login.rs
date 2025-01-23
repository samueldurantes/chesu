use crate::{
    http::{
        error::{Error, GenericError},
        extractor::COOKIE_NAME,
        Result,
    },
    services::user::login_service::{LoginInput, LoginUserService},
};
use crate::{models::user::User, repositories::user_repository::UserRepository};
use aide::transform::TransformOperation;
use axum::{
    http::{header::SET_COOKIE, HeaderName},
    response::AppendHeaders,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn build_set_cookie(token: Option<String>) -> String {
    let cookie = Cookie::build((COOKIE_NAME, token.unwrap_or_default()))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict);

    cookie.to_string()
}

pub fn service() -> LoginUserService<UserRepository> {
    LoginUserService::new(UserRepository::new())
}

pub async fn route(
    user_login_service: LoginUserService<UserRepository>,
    Json(payload): Json<UserBody<LoginUser>>,
) -> Result<(
    AppendHeaders<[(HeaderName, String); 1]>,
    Json<UserBody<User>>,
)> {
    if let Some(message) = validate_user_payload(&payload) {
        return Err(Error::BadRequest { message });
    }

    let (user, token) = user_login_service
        .execute(LoginInput {
            email: payload.user.email,
            password: payload.user.password,
        })
        .await?;

    Ok((
        AppendHeaders([(SET_COOKIE, build_set_cookie(Some(token)))]),
        Json(UserBody { user }),
    ))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Auth")
        .description("Login an user")
        .response::<200, Json<UserBody<User>>>()
        .response::<400, Json<GenericError>>()
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UserBody<T> {
    pub user: T,
}

#[derive(Validate, Deserialize, JsonSchema)]
pub struct LoginUser {
    #[validate(email(message = "Invalid email"))]
    email: String,
    password: String,
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
