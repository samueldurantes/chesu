use crate::states::db;

use crate::http::{Error, Result};
use sqlx::prelude::FromRow;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::user::User;

pub struct SaveUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(FromRow)]
struct ReturnedId {
    id: Uuid,
}

#[derive(FromRow)]
struct ReturnedUser {
    id: Uuid,
    username: String,
    email: String,
    password: String,
}

#[derive(FromRow)]
struct ReturnedLastBalance {
    last_balance: i32,
}

pub trait UserRepositoryTrait {
    async fn find_by_email(&self, email: String) -> sqlx::Result<User>;
    async fn find_by_id(&self, id: Uuid) -> sqlx::Result<User>;
    async fn save(&self, user: SaveUser) -> Result<Uuid>;
}

pub struct UserRepository {
    db: Pool<Postgres>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self { db: db::get() }
    }
}

impl UserRepositoryTrait for UserRepository {
    async fn find_by_email(&self, email: String) -> sqlx::Result<User> {
        let ReturnedUser {
            id,
            username,
            email,
            password: hashed_password,
        } = sqlx::query_as::<_, ReturnedUser>(
            r#" SELECT id, username, email, password FROM users WHERE email = $1 "#,
        )
        .bind(email)
        .fetch_one(&self.db)
        .await?;

        let ReturnedLastBalance { last_balance: balance } = sqlx::query_as::<_, ReturnedLastBalance>(
            r#" SELECT last_balance FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#,
        )
            .bind(id)
        .fetch_one(&self.db)
        .await?;

        Ok(User {
            id,
            username,
            email,
            hashed_password,
            balance,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> sqlx::Result<User> {
        let ReturnedUser {
            id,
            username,
            email,
            password: hashed_password,
        } = sqlx::query_as::<_, ReturnedUser>(
            r#" SELECT id, username, email, password FROM users WHERE id = $1 "#,
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        let ReturnedLastBalance { last_balance: balance } = sqlx::query_as::<_, ReturnedLastBalance>(
            r#" SELECT last_balance FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#,
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        Ok(User {
            id,
            username,
            email,
            balance,
            hashed_password,
        })
    }

    async fn save(&self, user: SaveUser) -> Result<Uuid> {
        let ReturnedId { id } = sqlx::query_as::<_, ReturnedId>(
            r#" INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id "#,
        )
        .bind(user.username)
        .bind(user.email)
        .bind(user.password_hash)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return Error::BadRequest {
                        message: String::from("Already exists an user with these credentials"),
                    };
                }
            }

            return Error::Anyhow(e.into());
        })?;

        sqlx::query(r#" INSERT INTO transactions (user_id, type, amount, last_balance) VALUES ( $1, 'input', 0, 0); "#)
        .bind(id)
        .execute(&self.db).await?;

        Ok(id)
    }
}
