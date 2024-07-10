use anyhow::Result;
use dotenvy::dotenv;
use server::AppState;
use sqlx::postgres::PgPoolOptions;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = &std::env::var("DATABASE_URL")?;

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(database_url)
        .await?;

    sqlx::migrate!().run(&db).await?;

    let state = AppState {
        db,
        rooms: Arc::new(Mutex::new(HashMap::new())),
    };

    let (app, _) = server::app::make_app()?;
    let app = app.with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    tracing::debug!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
