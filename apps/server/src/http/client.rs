use crate::http::error::Error;
use axum::async_trait;
use reqwest::Client;
use reqwest::Response;
use serde::Serialize;
use std::marker::Sync;

#[async_trait]
pub trait HttpClient {
    async fn post<T: Serialize + Sync>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, Error>;
}

#[async_trait]
impl HttpClient for Client {
    async fn post<T: Serialize + Sync>(&self, path: &str, body: &T) -> Result<Response, Error> {
        let token = &std::env::var("LSP_TOKEN").expect("LSP_TOKEN is void");

        self.post(format!("https://api.getalby.com{}", path))
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await
            .map_err(|_| crate::Error::BadRequest {
                message: String::from("Failed to send request"),
            })
    }
}
