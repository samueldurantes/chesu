use crate::http::Result;
use anyhow::anyhow;
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
        let token = &std::env::var("LSP_TOKEN").expect("LSP_TOKEN is void");
        self.post(format!("https://api.getalby.com{}", path))
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|_| crate::Error::Anyhow(anyhow!("Failed to send request")))
    }
}
