use crate::http::client::HttpClient;
use crate::http::{error::Error, extractor::AuthUser, Result};
use aide::transform::TransformOperation;
use axum::Json;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::http::error::GenericError;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AmountBody {
    amount: i32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InvoiceResponseBody {
    payment_request: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InvoiceBody {
    invoice: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InvoiceBuilder {
    amount: i32,
    memo: String,
}

fn resource() -> impl HttpClient {
    Client::new()
}

pub async fn route(
    auth_user: AuthUser,
    Json(payload): Json<AmountBody>,
) -> Result<Json<InvoiceBody>> {
    let client = resource();

    if payload.amount <= 0 {
        return Err(Error::BadRequest {
            message: String::from("Invalid invoice input"),
        });
    }

    let response = client
        .post(
            "/invoices",
            &InvoiceBuilder {
                amount: payload.amount,
                memo: auth_user.user_id.to_string(),
            },
        )
        .await?;

    if !response.status().is_success() {
        return Err(Error::BadRequest {
            message: String::from("Invalid invoice input"),
        });
    }

    let InvoiceResponseBody {
        payment_request: invoice,
    } = response.json().await.unwrap();

    return Ok(Json(InvoiceBody { invoice }));
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Create Deposit Invoice")
        .description("Create an invoice to deposit satoshis")
        .response::<200, Json<InvoiceBody>>()
        .response::<400, Json<GenericError>>()
}
