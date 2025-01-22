use std::str::FromStr;

use crate::http::error::Error;
use crate::http::{extractor::AuthUser, Result};
use aide::{
    axum::{routing::get_with, routing::post_with, ApiRouter},
    transform::TransformOperation,
};
use axum::extract::State;
use axum::Json;
use lightning_invoice::Bolt11Invoice;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::error::GenericError;

pub(crate) fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route(
            "/invoice/create",
            post_with(create_deposit_invoice, create_deposit_invoice_docs),
        )
        .api_route(
            "/invoice/settled",
            post_with(deposit_webhook_handler, deposit_webhook_handler_docs),
        )
        .api_route(
            "/invoice/check",
            get_with(check_invoice, check_invoice_docs),
        )
        .api_route("/invoice/withdraw", post_with(withdraw, withdraw_docs))
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InvoiceBody {
    invoice: String,
}

struct InvoiceField {
    invoice: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct AmountBody {
    amount: i32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct BalanceBody {
    balance: i32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InvoiceBuilder {
    amount: i32,
    memo: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InvoiceSettled {
    payment_request: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct InvoiceResponseBody {
    payment_request: String,
}

async fn check_invoice(
    auth_user: AuthUser,
    state: State<crate::AppState>,
) -> Result<Json<InvoiceBody>> {
    let result = sqlx::query_as!(
        InvoiceField,
        r#" SELECT (invoice) FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#,
        auth_user.user_id
    )
    .fetch_optional(&state.db)
    .await?.unwrap_or( InvoiceField { invoice: None });

    Ok(Json(InvoiceBody {
        invoice: result.invoice.unwrap_or(String::from("")),
    }))
}

fn check_invoice_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Check Invoice")
        .description("Check invoice payment")
        .response::<200, Json<InvoiceBody>>()
        .response::<400, Json<GenericError>>()
}

async fn withdraw(
    auth_user: AuthUser,
    state: State<crate::AppState>,
    Json(payload): Json<InvoiceBody>,
) -> Result<()> {
    let amount = (Bolt11Invoice::from_str(&payload.invoice)
        .map_err(|_| Error::BadRequest {
            message: String::from("Invalid invoice input"),
        })?
        .amount_milli_satoshis()
        .unwrap_or(0)
        / 1000) as i32;

    let last_balance = sqlx::query_scalar!(
        r#" SELECT last_balance FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1 "#,
        auth_user.user_id
    )
    .fetch_one(&state.db)
    .await?;

    if amount <= 0 || amount > last_balance {
        return Err(Error::BadRequest {
            message: String::from("Invalid invoice input"),
        });
    }

    struct TransacionID {
        id: uuid::Uuid,
    }

    let TransacionID { id } = sqlx::query_as!(
        TransacionID,
        r#"
            WITH last_transaction AS (
                SELECT last_balance
                FROM transactions
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT 1
            )
            INSERT INTO transactions (user_id, type, amount, last_balance)
            VALUES ($1, 'output', $2, (SELECT last_balance FROM last_transaction) - $2)
            RETURNING id;
    "#,
        auth_user.user_id,
        amount
    )
    .fetch_one(&state.db)
    .await?;

    let client = Client::new();
    let token = format!(
        "Bearer {}",
        &std::env::var("LSP_TOKEN").expect("LSP_TOKEN is void")
    );

    let response = client
        .post("https://api.getalby.com/payments/bolt11")
        .header("Authorization", token)
        .json(&payload)
        .send()
        .await
        .map_err(|_| Error::BadRequest {
            message: String::from("Invalid invoice input"),
        })?;

    if !response.status().is_success() {
        sqlx::query!(r#" DELETE FROM transactions WHERE id = $1 "#, id)
            .execute(&state.db)
            .await?;

        return Err(Error::BadRequest {
            message: String::from("Invalid invoice input"),
        });
    }

    Ok(())
}

fn withdraw_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Deposit Webhook Handler")
        .description("Confirms deposit payment")
        .response::<200, ()>()
        .response::<400, Json<GenericError>>()
}

// TODO: Set cors to lsp origin
async fn deposit_webhook_handler(
    state: State<crate::AppState>,
    Json(payload): Json<InvoiceSettled>,
) -> Result<()> {
    let invoice = Bolt11Invoice::from_str(&payload.payment_request).unwrap();

    let user_id = uuid::Uuid::from_str(&invoice.description().to_string()).unwrap();
    let amount = (invoice.amount_milli_satoshis().unwrap() / 1000) as i32;

    sqlx::query!(
        r#"
            WITH last_transaction AS (
                SELECT last_balance AS last_balance
                FROM transactions
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT 1
            )
            INSERT INTO transactions (user_id, type, amount, last_balance, invoice)
            VALUES ( $1, 'input', $2, (SELECT last_balance FROM last_transaction) + $2, $3);
        "#,
        user_id,
        amount,
        payload.payment_request
    )
    .execute(&state.db)
    .await?;

    Ok(())
}

fn deposit_webhook_handler_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Deposit Webhook Handler")
        .description("Confirms deposit")
        .response::<200, ()>()
        .response::<400, Json<GenericError>>()
}

async fn create_deposit_invoice(
    auth_user: AuthUser,
    _: State<crate::AppState>,
    Json(payload): Json<AmountBody>,
) -> Result<Json<InvoiceBody>> {
    if payload.amount <= 0 {
        return Err(Error::BadRequest {
            message: String::from("Invalid invoice input"),
        });
    }

    let invoice_builder = InvoiceBuilder {
        amount: payload.amount,
        memo: auth_user.user_id.to_string(),
    };

    let client = Client::new();
    let token = format!(
        "Bearer {}",
        &std::env::var("LSP_TOKEN").expect("LSP_TOKEN is void")
    );

    let response = client
        .post("https://api.getalby.com/invoices")
        .header("Authorization", token)
        .json(&invoice_builder)
        .send()
        .await
        .map_err(|_| Error::BadRequest {
            message: String::from("Invalid invoice input"),
        })?;

    if !response.status().is_success() {
        return Err(Error::BadRequest {
            message: String::from("Invalid invoice input"),
        });
    }

    let invoice_response: InvoiceResponseBody = response.json().await.unwrap();
    let invoice = invoice_response.payment_request;

    return Ok(Json(InvoiceBody { invoice }));
}

fn create_deposit_invoice_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Create Deposit Invoice")
        .description("Create an invoice to deposit satoshis")
        .response::<200, Json<InvoiceBody>>()
        .response::<400, Json<GenericError>>()
}
