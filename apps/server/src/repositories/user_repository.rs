use crate::http::{Error, Result};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::user::User;

pub struct SaveUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

struct ReturnedId {
    id: Uuid,
}

struct ReturnedUser {
    id: Uuid,
    username: String,
    email: String,
}

struct ReturnedLastBalance {
    last_balance: i32,
}

pub trait UserRepositoryTrait {
    async fn _find_by_id(&self, id: Uuid) -> sqlx::Result<User>;
    async fn save(&self, user: SaveUser) -> Result<Uuid>;
}

pub struct UserRepository {
    db: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }
}

impl UserRepositoryTrait for UserRepository {
    async fn _find_by_id(&self, id: Uuid) -> sqlx::Result<User> {
        let ReturnedLastBalance { last_balance: balance } = sqlx::query_as!(
            ReturnedLastBalance,
            r#" SELECT last_balance FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#,
            id
        )
        .fetch_one(&self.db)
        .await?;

        let ReturnedUser {
            id,
            username,
            email,
        } = sqlx::query_as!(
            ReturnedUser,
            r#" SELECT id, username, email FROM users WHERE id = $1 "#,
            id,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(User {
            id,
            username,
            email,
            balance,
        })
    }

    async fn save(&self, user: SaveUser) -> Result<Uuid> {
        let ReturnedId { id } = sqlx::query_as!(
            ReturnedId,
            r#" INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id "#,
            user.username,
            user.email,
            user.password_hash,
        )
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

        sqlx::query!(r#" INSERT INTO transactions (user_id, type, amount, last_balance) VALUES ( $1, 'input', 0, 0); "#, id)
            .execute(&self.db).await?;

        Ok(id)
    }
}
