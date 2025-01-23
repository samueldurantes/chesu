use std::sync::Arc;

use crate::http::Result;
use aide::{
    axum::{
        routing::{get, get_with},
        ApiRouter,
    },
    openapi::OpenApi,
    redoc::Redoc,
};
use axum::{Extension, Json};

pub fn router() -> ApiRouter<crate::AppState> {
    aide::gen::infer_responses(true);

    let router = ApiRouter::new()
        .api_route_with(
            "/docs",
            get_with(
                Redoc::new("/docs/private/api.json")
                    .with_title("Chesu API")
                    .axum_handler(),
                |op| op.hidden(true),
            ),
            |p| p.security_requirement("ApiKey"),
        )
        .route("/docs/private/api.json", get(serve_docs));

    aide::gen::infer_responses(false);

    router
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> Result<Json<Arc<OpenApi>>> {
    Ok(Json(api))
}
