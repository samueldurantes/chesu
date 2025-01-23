use server::{app::make_app, AppState};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path(server::app::get_dotenv_path()).expect(".env file not found");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL is void"))
        .await?;

    sqlx::migrate!().run(&db).await?;

    let (app, _) = make_app(db.clone());

    let listener =
        tokio::net::TcpListener::bind(&std::env::var("SERVER_URL").expect("SERVER_URL is void"))
            .await
            .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.with_state(AppState::new(db)))
        .await
        .unwrap();

    Ok(())
}
