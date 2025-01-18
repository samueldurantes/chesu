use dotenvy::dotenv;
use server::AppState;
use sqlx::postgres::PgPoolOptions;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().expect(".env file not found");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL is void"))
        .await?;

    sqlx::migrate!().run(&db).await?;

    let state = AppState {
        db,
        rooms: Arc::new(Mutex::new(HashMap::new())),
        pairing_room: Arc::new(server::PairingRoom {
            game: Mutex::new(None),
            notifier: Notify::new(),
        }),
    };

    let (app, _) = server::app::make_app();
    let app = app.with_state(state);

    let listener =
        tokio::net::TcpListener::bind(&std::env::var("SERVER_URL").expect("SERVER_URL is void"))
            .await
            .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
