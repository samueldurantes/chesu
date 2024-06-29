use crate::http::{error::Error, extractor::AuthUser, Result};
use aide::{
    axum::{routing::post_with, ApiRouter},
    transform::TransformOperation,
};
use anyhow::Context;
use argon2::{password_hash::SaltString, Argon2, PasswordHash};
use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderName},
    response::AppendHeaders,
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

pub(crate) fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route("/auth/register", post_with(register, register_docs))
        .api_route("/auth/login", post_with(login, login_docs))
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
    username: String,
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

fn mount_set_cookie(token: String) -> String {
    format!("CHESU_TOKEN={}", token)
}

async fn register(
    state: State<crate::AppState>,
    Json(payload): Json<UserBody<RegisterUser>>,
) -> Result<(
    AppendHeaders<[(HeaderName, String); 1]>,
    Json<UserBody<User>>,
)> {
    if let Some(error) = validate_user_payload(&payload) {
        return Err(Error::BadRequest { error });
    }

    let password_hash = hash_password(payload.user.password).await?;
    let user = sqlx::query_scalar!(
        r#"
            INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id
        "#,
        payload.user.username,
        payload.user.email,
        password_hash,
    )
    .fetch_one(&state.db)
    .await;

    match user {
        Ok(user_id) => {
            let token = AuthUser { user_id }.to_jwt();

            Ok((
                AppendHeaders([(SET_COOKIE, mount_set_cookie(token))]),
                Json(UserBody {
                    user: User {
                        id: user_id.to_string(),
                        username: payload.user.username,
                        email: payload.user.email,
                    },
                }),
            ))
        }
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return Err(Error::BadRequest {
                        error: "Already exists an user with these credentials".to_string(),
                    });
                }
            }

            return Err(Error::Anyhow(e.into()));
        }
    }
}

fn register_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Auth")
        .description("Register an user")
        .response::<200, Json<UserBody<User>>>()
}

async fn login(
    state: State<crate::AppState>,
    Json(payload): Json<UserBody<LoginUser>>,
) -> Result<(
    AppendHeaders<[(HeaderName, String); 1]>,
    Json<UserBody<User>>,
)> {
    if let Some(error) = validate_user_payload(&payload) {
        return Err(Error::BadRequest { error });
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
        error: "User not found".to_string(),
    })?;

    verify_password(payload.user.password, user.password).await?;

    let token = AuthUser { user_id: user.id }.to_jwt();

    Ok((
        AppendHeaders([(SET_COOKIE, mount_set_cookie(token))]),
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
}

async fn verify_password(password: String, password_hash: String) -> Result<()> {
    tokio::task::spawn_blocking(move || -> Result<()> {
        let hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

        hash.verify_password(&[&Argon2::default()], password)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => Error::BadRequest {
                    error: "Email or password incorrect".to_string(),
                },
                _ => anyhow::anyhow!("Failed to verify password hash: {}", e).into(),
            })
    })
    .await
    .context("Panic in verifying password hash")?
}

async fn hash_password(password: String) -> Result<String> {
    // Argon2 is compute-intesive, so we run it in a blocking task
    tokio::task::spawn_blocking(move || -> Result<String> {
        let salt = SaltString::generate(rand::thread_rng());
        let password_hash = PasswordHash::generate(Argon2::default(), password, salt.as_salt())
            .map_err(|e| anyhow::anyhow!("Failed to generate password hash: {}", e))?
            .to_string();

        Ok(password_hash)
    })
    .await
    .context("Panic in generating password hash")?
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
