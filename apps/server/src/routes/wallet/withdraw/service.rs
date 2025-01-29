use crate::bad_req;
use crate::http::client::HttpClient;
use crate::http::{Error, Result};
use crate::repositories::wallet_repository::{SaveOutgoing, WalletRepositoryTrait};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct WithdrawService<R: WalletRepositoryTrait, C: HttpClient> {
    wallet_repository: R,
    client: C,
}

pub struct WithdrawInput {
    pub user_id: Uuid,
    pub amount: i32,
    pub invoice: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InvoiceBody {
    invoice: String,
}

impl<R: WalletRepositoryTrait, C: HttpClient> WithdrawService<R, C> {
    pub fn new(wallet_repository: R, client: C) -> Self {
        Self {
            wallet_repository,
            client,
        }
    }

    pub async fn execute(
        &self,
        WithdrawInput {
            user_id,
            amount,
            invoice,
        }: WithdrawInput,
    ) -> Result<()> {
        let balance = self.wallet_repository.get_balance(user_id).await?;

        if amount <= 0 || amount > balance {
            return bad_req!("Invalid invoice input");
        }

        let response = self
            .client
            .post("/payments/bolt11", &InvoiceBody { invoice })
            .await?;

        if !response.status().is_success() {
            return Err(bad_request());
        }

        self.wallet_repository
            .save_outgoing(SaveOutgoing { user_id, amount })
            .await?;

        Ok(())
    }
}
