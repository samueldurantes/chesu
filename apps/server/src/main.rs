use server::db::get_db;
use server::db::init_db;
use server::{app::make_app, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path(server::app::get_dotenv_path()).expect(".env file not found");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    init_db().await;

    sqlx::migrate!().run(&*get_db()).await?;

    let (app, _) = make_app();

    let listener =
        tokio::net::TcpListener::bind(&std::env::var("SERVER_URL").expect("SERVER_URL is void"))
            .await
            .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.with_state(AppState::new(get_db())))
        .await
        .unwrap();

    Ok(())
}
