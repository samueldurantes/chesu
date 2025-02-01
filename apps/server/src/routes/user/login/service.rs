use crate::http::{extractor::AuthUser, Error, Result};
use crate::models::user::User;
use crate::repositories::user_repository::UserRepositoryTrait;
use anyhow::Context;
use argon2::{Argon2, PasswordHash};

pub struct LoginUserService<R: UserRepositoryTrait> {
    user_repository: R,
}

pub struct LoginInput {
    pub email: String,
    pub password: String,
}

impl<R: UserRepositoryTrait> LoginUserService<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }

    pub async fn execute(
        &self,
        LoginInput { email, password }: LoginInput,
    ) -> Result<(User, String)> {
        let user = self
            .user_repository
            .find_by_email(email)
            .await
            .map_err(|_| Error::BadRequest {
                message: "Email and/or password incorrect".to_string(),
            })?;

        verify_password(password, &user.hashed_password).await?;

        let token = AuthUser { user_id: user.id }.to_jwt();

        Ok((user, token))
    }
}

async fn verify_password(password: String, password_hash: &String) -> Result<()> {
    let password_hash = password_hash.to_string();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

        hash.verify_password(&[&Argon2::default()], password)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => crate::http::Error::BadRequest {
                    message: "Email and/or password incorrect".to_string(),
                },
                _ => anyhow::anyhow!("Failed to verify password hash: {}", e).into(),
            })
    })
    .await
    .context("Panic in verifying password hash")?
}
