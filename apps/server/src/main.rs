use dotenvy::dotenv;
use server::AppState;
use sqlx::postgres::PgPoolOptions;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenv().expect(".env file not found");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // This connection is shared in whole application
    let db = PgPoolOptions::new()
        .max_connections(50)
        // TODO: Improve this to show when DATABASE_URL is void
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await?;

    sqlx::migrate!().run(&db).await?;

    let state = AppState {
        db,
        rooms: Arc::new(Mutex::new(HashMap::new())),
    };

    let (app, _) = server::app::make_app();
    let app = app.with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
