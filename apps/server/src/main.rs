use std::time::Duration;

use axum::{http::header::AUTHORIZATION, routing::{post}, Router};
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer,
    sensitive_headers::SetSensitiveHeadersLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod http;

#[derive(Clone)]
pub struct Context {
    pub db: PgPool,
}

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
        .await
        .unwrap();

    sqlx::migrate!().run(&db).await?;

    let context = Context { db };

    let app = Router::new()
        .route("/auth/register", post(http::auth::register))
        .route("/auth/login", post(http::auth::login))
        .layer((
            SetSensitiveHeadersLayer::new([AUTHORIZATION]),
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
        ))
        .with_state(context);

    let listener = tokio::net::TcpListener::bind("::1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
