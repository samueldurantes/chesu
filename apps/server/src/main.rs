use server::states::db;
use server::{app::make_app, Env};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Env::init();
    db::init().await;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    sqlx::migrate!().run(&db::get()).await?;

    let listener = tokio::net::TcpListener::bind(&Env::get().server_url).await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, make_app()).await?;

    Ok(())
}
