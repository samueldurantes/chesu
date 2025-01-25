use server::states::db;
use server::{app::make_app, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path(server::app::get_dotenv_path()).expect(".env file not found");

    db::init().await;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    sqlx::migrate!().run(&*db::get()).await?;

    let (app, _) = make_app();

    let listener =
        tokio::net::TcpListener::bind(&std::env::var("SERVER_URL").expect("SERVER_URL is void"))
            .await
            .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.with_state(AppState::new(db::get())))
        .await
        .unwrap();

    Ok(())
}
