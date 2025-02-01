use crate::http::Result;
use crate::states::db;
use mockall::automock;
use sqlx::{prelude::FromRow, Pool, Postgres};
use uuid::Uuid;

pub struct SaveIncoming {
    pub user_id: Uuid,
    pub amount: i32,
    pub invoice: Option<String>,
}

pub struct SaveOutgoing {
    pub user_id: Uuid,
    pub amount: i32,
}

struct SaveTransaction {
    user_id: Uuid,
    amount: i32,
    direction: String,
    invoice: Option<String>,
}

#[derive(FromRow)]
struct ReturnedId {
    id: Uuid,
}

#[derive(FromRow)]
struct ReturnedInvoice {
    invoice: Option<String>,
}

#[automock]
pub trait WalletRepositoryTrait {
    async fn save_incoming(&self, info: SaveIncoming) -> Result<Uuid>;
    async fn save_outgoing(&self, info: SaveOutgoing) -> Result<Uuid>;
    async fn get_balance(&self, user_id: Uuid) -> Result<i32>;
    async fn get_invoice(&self, user_id: Uuid) -> Result<String>;
}

pub struct WalletRepository {
    db: Pool<Postgres>,
}

impl WalletRepository {
    pub fn new() -> Self {
        Self { db: db::get() }
    }
}

async fn save_transaction(db: &Pool<Postgres>, info: SaveTransaction) -> sqlx::Result<Uuid> {
    let ReturnedId { id } = sqlx::query_as::<_, ReturnedId>(
        r#"
            WITH last_transaction AS (
                SELECT last_balance AS last_balance
                FROM transactions
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT 1
            )
            INSERT INTO transactions (user_id, type, amount, last_balance, invoice)
            VALUES ( $1, $2, $3, (SELECT last_balance FROM last_transaction) + $3, $4) 
            RETURNING id;
        "#,
    )
    .bind(info.user_id)
    .bind(info.direction)
    .bind(info.amount)
    .bind(info.invoice.unwrap_or_default())
    .fetch_one(db)
    .await?;

    Ok(id)
}

impl WalletRepositoryTrait for WalletRepository {
    async fn save_incoming(&self, info: SaveIncoming) -> Result<Uuid> {
        let SaveIncoming {
            user_id,
            amount,
            invoice,
        } = info;

        Ok(save_transaction(
            &self.db,
            SaveTransaction {
                user_id,
                direction: String::from("input"),
                amount,
                invoice,
            },
        )
        .await?)
    }

    async fn save_outgoing(&self, info: SaveOutgoing) -> Result<Uuid> {
        let SaveOutgoing { user_id, amount } = info;

        Ok(save_transaction(
            &self.db,
            SaveTransaction {
                user_id,
                direction: String::from("output"),
                amount,
                invoice: None,
            },
        )
        .await?)
    }

    async fn get_balance(&self, user_id: Uuid) -> Result<i32> {
        Ok(sqlx::query_scalar(
            r#" SELECT last_balance FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?)
    }

    async fn get_invoice(&self, user_id: Uuid) -> Result<String> {
        Ok(sqlx::query_as::<_, ReturnedInvoice>(
            r#" SELECT invoice FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?.invoice.unwrap_or_default())
    }
}
