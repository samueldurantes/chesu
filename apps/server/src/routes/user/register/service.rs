use crate::http::{extractor::AuthUser, Result};
use crate::models::user::User;
use crate::repositories::user_repository::{SaveUser, UserRepositoryTrait};
use anyhow::Context;
use argon2::{password_hash::SaltString, Argon2, PasswordHash};

#[derive(Clone)]
pub struct RegisterUserService<R: UserRepositoryTrait> {
    user_repository: R,
}

pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl<R: UserRepositoryTrait> RegisterUserService<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }

    pub async fn execute(
        &self,
        RegisterInput {
            username,
            email,
            password,
        }: RegisterInput,
    ) -> Result<(User, String)> {
        let password_hash = hash_password(password).await?;

        let user_id = self
            .user_repository
            .save(SaveUser {
                username: username.clone(),
                email: email.clone(),
                password_hash: password_hash.clone(),
            })
            .await?;

        let token = AuthUser { user_id }.to_jwt();

        Ok((
            User {
                id: user_id,
                username,
                email,
                hashed_password: password_hash,
                balance: 0,
            },
            token,
        ))
    }
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
