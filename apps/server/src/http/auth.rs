use anyhow::Context;
use argon2::{password_hash::SaltString, Argon2, PasswordHash};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use crate::http::{error::Error, extractor::AuthUser, Result};
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct UserBody<T> {
    pub user: T,
}

#[derive(Validate, Deserialize)]
pub struct LoginUser {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    pub password: String,
}

#[derive(Validate, Deserialize)]
pub struct RegisterUser {
    pub username: String,
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub token: String,
}

pub async fn register(
    context: State<crate::Context>,
    Json(payload): Json<UserBody<RegisterUser>>
) -> Result<Json<UserBody<User>>> {
    if let Some(error) = validate_user_payload(&payload) {
        return Err(Error::BadRequest {
            error,
        });
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
    .fetch_one(&context.db)
    .await;

    match user {
        Ok(user_id) => {
            let token = AuthUser { user_id }.to_jwt();

            Ok(Json(UserBody {
                user: User {
                    id: user_id.to_string(),
                    username: payload.user.username,
                    email: payload.user.email,
                    token,
                }
            }))
        },
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return Err(Error::BadRequest {
                        error: "Already exists an user with these credentials".to_string(),
                    });
                }
            }

            return Err(Error::Anyhow(e.into()))
        }
    }
}

pub async fn login(
    context: State<crate::Context>,
    Json(payload): Json<UserBody<LoginUser>>,
) -> Result<Json<UserBody<User>>> {
    if let Some(error) = validate_user_payload(&payload) {
        return Err(Error::BadRequest {
            error,
        });
    }

    let user = sqlx::query!(
        r#"
            SELECT id, username, email, password FROM users WHERE email = $1
        "#,
        payload.user.email,
    )
    .fetch_optional(&context.db)
    .await?
    .ok_or_else(|| Error::BadRequest {
        error: "User not found".to_string(),
    })?;

    verify_password(payload.user.password, user.password).await?;

    let token = AuthUser { user_id: user.id }.to_jwt();

    Ok(Json(UserBody {
        user: User {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            token,
        }
    }))
}

async fn verify_password(password: String, password_hash: String) -> Result<()> {
    tokio::task::spawn_blocking(move || -> Result<()> {
        let hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
        
        hash.verify_password(&[&Argon2::default()], password)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => {
                    Error::BadRequest {
                        error: "Email or password incorrect".to_string(),
                    }
                },
                _ => anyhow::anyhow!("Failed to verify password hash: {}", e).into()
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
