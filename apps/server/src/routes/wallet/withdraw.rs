use std::str::FromStr;

use crate::http::error::Error;
use crate::http::{extractor::AuthUser, Result};
use crate::repositories::wallet_repository::WalletRepository;
use crate::services::withdraw_service::{WithdrawInput, WithdrawService};
use aide::transform::TransformOperation;
use axum::Json;
use lightning_invoice::Bolt11Invoice;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::http::error::GenericError;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InvoiceBody {
    invoice: String,
}

pub fn resource() -> WithdrawService<WalletRepository, Client> {
    WithdrawService::new(WalletRepository::new(), Client::new())
}

pub async fn route(
    withdraw_service: WithdrawService<WalletRepository, Client>,
    auth_user: AuthUser,
    Json(payload): Json<InvoiceBody>,
) -> Result<()> {
    let amount = (Bolt11Invoice::from_str(&payload.invoice)
        .map_err(|_| Error::BadRequest {
            message: String::from("Invalid invoice input"),
        })?
        .amount_milli_satoshis()
        .unwrap_or(0)
        / 1000) as i32;

    withdraw_service
        .execute(WithdrawInput {
            user_id: auth_user.user_id,
            amount,
            invoice: payload.invoice,
        })
        .await?;

    Ok(())
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Deposit Webhook Handler")
        .description("Confirms deposit payment")
        .response::<200, ()>()
        .response::<400, Json<GenericError>>()
}
