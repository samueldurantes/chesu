use std::{sync::Arc, time::Duration};

use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    Extension,
};
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod http;

#[derive(Clone)]
pub struct State {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    aide::gen::on_error(|error| {
        println!("{error}");
    });

    aide::gen::extract_schemas(true);

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

    let state = State { db };

    let mut api = OpenApi::default();
    let app = ApiRouter::new()
        // Auth routes
        .merge(http::auth::router())
        // Docs routes
        .merge(http::docs::router())
        // Game routes
        .merge(http::game::router())
        // User routes
        .merge(http::user::router())
        .finish_api_with(&mut api, api_docs)
        .layer((
            SetSensitiveHeadersLayer::new([AUTHORIZATION]),
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
            Extension(Arc::new(api)),
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([CONTENT_TYPE])
                .allow_origin(Any),
        ))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("::1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Chesu API")
        .description("Chesu API Documentation")
}
