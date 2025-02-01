use crate::http::Result;
use crate::repositories::{SaveIncoming, WalletRepository, WalletRepositoryTrait};
use aide::transform::TransformOperation;
use axum::Json;
use lightning_invoice::Bolt11Invoice;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::http::GenericError;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InvoiceSettled {
    payment_request: String,
}

fn resource() -> WalletRepository {
    WalletRepository::new()
}

// TODO: Set cors to lsp origin
pub async fn route(Json(payload): Json<InvoiceSettled>) -> Result<()> {
    let wallet_repository = resource();

    let invoice = Bolt11Invoice::from_str(&payload.payment_request).unwrap();
    let user_id = uuid::Uuid::from_str(&invoice.description().to_string()).unwrap();
    let amount = (invoice.amount_milli_satoshis().unwrap() / 1000) as i32;

    wallet_repository
        .save_incoming(SaveIncoming {
            user_id,
            invoice: Some(invoice.to_string()),
            amount,
        })
        .await?;

    Ok(())
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Deposit Webhook Handler")
        .description("Confirms deposit")
        .response::<200, ()>()
        .response::<400, Json<GenericError>>()
}
