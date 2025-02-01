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

        self.client
            .post("/payments/bolt11", &InvoiceBody { invoice })
            .await?;

        self.wallet_repository
            .save_outgoing(SaveOutgoing { user_id, amount })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::wallet_repository::MockWalletRepositoryTrait;
    use reqwest::Client;
    use uuid::uuid;

    #[tokio::test]
    async fn test_withdraw_service_1() {
        let mut mock_wallet_repository = MockWalletRepositoryTrait::new();

        mock_wallet_repository
            .expect_get_balance()
            .once()
            .withf(|id| id == &uuid!("55bc0856-6b5a-4e5a-b294-bf82921a996a"))
            .returning(|_| Ok(1000));

        let input = WithdrawInput {
            user_id: uuid!("55bc0856-6b5a-4e5a-b294-bf82921a996a"),
            invoice: String::new(),
            amount: 2000,
        };

        let service = WithdrawService::new(mock_wallet_repository, Client::new());

        let result = service.execute(input).await.ok();

        assert!(result.is_none())
    }

    #[tokio::test]
    async fn test_withdraw_service_2() {
        let mut mock_wallet_repository = MockWalletRepositoryTrait::new();

        mock_wallet_repository
            .expect_get_balance()
            .once()
            .withf(|id| id == &uuid!("55bc0856-6b5a-4e5a-b294-bf82921a996a"))
            .returning(|_| Ok(1000));

        let input = WithdrawInput {
            user_id: uuid!("55bc0856-6b5a-4e5a-b294-bf82921a996a"),
            invoice: String::new(),
            amount: -2000,
        };

        let service = WithdrawService::new(mock_wallet_repository, Client::new());

        let result = service.execute(input).await.ok();

        assert!(result.is_none())
    }
}
