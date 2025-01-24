use crate::http::{extractor::AuthUser, Result};
use crate::repositories::wallet_repository::{WalletRepository, WalletRepositoryTrait};
use aide::transform::TransformOperation;
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::http::error::GenericError;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InvoiceBody {
    invoice: String,
}

pub fn resource() -> WalletRepository {
    WalletRepository::new()
}

pub async fn route(
    wallet_repository: WalletRepository,
    auth_user: AuthUser,
) -> Result<Json<InvoiceBody>> {
    let invoice = wallet_repository.get_invoice(auth_user.user_id).await?;
    Ok(Json(InvoiceBody { invoice }))
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Check Invoice")
        .description("Check invoice payment")
        .response::<200, Json<InvoiceBody>>()
        .response::<400, Json<GenericError>>()
}
