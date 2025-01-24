use sqlx::{Pool, Postgres};
use std::sync::Arc;
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

struct ReturnedId {
    id: Uuid,
}

struct ReturnedInvoice {
    invoice: Option<String>,
}

pub trait WalletRepositoryTrait {
    async fn save_incoming(&self, info: SaveIncoming) -> sqlx::Result<Uuid>;
    async fn save_outgoing(&self, info: SaveOutgoing) -> sqlx::Result<Uuid>;
    async fn get_balance(&self, user_id: Uuid) -> sqlx::Result<i32>;
    async fn get_invoice(&self, user_id: Uuid) -> sqlx::Result<String>;
}

pub struct WalletRepository {
    db: Arc<Pool<Postgres>>,
}

impl WalletRepository {
    pub fn new() -> Self {
        let db = crate::db::get_db();
        Self { db }
    }
}

async fn save_transaction(db: &Pool<Postgres>, info: SaveTransaction) -> sqlx::Result<Uuid> {
    let ReturnedId { id } = sqlx::query_as!(
        ReturnedId,
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
        info.user_id,
        info.direction,
        info.amount,
        info.invoice.unwrap_or_default()
    )
    .fetch_one(&*db)
    .await?;

    Ok(id)
}

impl WalletRepositoryTrait for WalletRepository {
    async fn save_incoming(&self, info: SaveIncoming) -> sqlx::Result<Uuid> {
        let SaveIncoming {
            user_id,
            amount,
            invoice,
        } = info;

        save_transaction(
            &*self.db,
            SaveTransaction {
                user_id,
                direction: String::from("input"),
                amount,
                invoice,
            },
        )
        .await
    }

    async fn save_outgoing(&self, info: SaveOutgoing) -> sqlx::Result<Uuid> {
        let SaveOutgoing { user_id, amount } = info;

        save_transaction(
            &*self.db,
            SaveTransaction {
                user_id,
                direction: String::from("output"),
                amount,
                invoice: None,
            },
        )
        .await
    }

    async fn get_balance(&self, user_id: Uuid) -> sqlx::Result<i32> {
        sqlx::query_scalar!(
            r#" SELECT last_balance FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#,
            user_id
        )
        .fetch_one(&*self.db)
        .await
    }

    async fn get_invoice(&self, user_id: Uuid) -> sqlx::Result<String> {
        Ok(sqlx::query_as!(
            ReturnedInvoice,
            r#" SELECT invoice FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#,
            user_id
        )
        .fetch_one(&*self.db)
        .await?.invoice.unwrap_or_default())
    }
}
