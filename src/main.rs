use gangrennes::{build_app, config::AppConfig, utils::error::AppResult, utils::shutdown::shutdown_signal};

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = AppConfig::from_env()?;
    let app = build_app(&config).await?;
    let listener = tokio::net::TcpListener::bind(config.bind_address()).await?;

    tracing::info!(address = %config.bind_address(), "server started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,gangrennes=debug".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}
