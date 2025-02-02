use crate::{http::Result, Env};
use axum::async_trait;
use reqwest::Client;
use reqwest::Response;
use serde::Serialize;
use std::marker::Sync;

#[async_trait]
pub trait HttpClient {
    async fn post<T: Serialize + Sync>(&self, path: &str, body: &T) -> Result<Response>;
}

#[async_trait]
impl HttpClient for Client {
    async fn post<T: Serialize + Sync>(&self, path: &str, body: &T) -> Result<Response> {
        self.post(format!("https://api.getalby.com{}", path))
            .header("Authorization", format!("Bearer {}", &Env::get().lsp_token))
            .json(body)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|_| crate::Error::InternalServerError)
    }
}
