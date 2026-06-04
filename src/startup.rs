use crate::{api, config::Config, db, state::AppState};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber
        ::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    tokio::fs::create_dir_all("uploads").await?;
    let pool = db::create_pool(&config.db.database_url).await?;
    db::run_migrations(&pool).await?;

    let state = AppState::new(config.clone(), pool);
    let router = api::build_router(state);

    let listener = tokio::net::TcpListener::bind(config.app.bind_addr).await?;
    tracing::info!(address = %config.app.bind_addr, "server listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                let _ = stream.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}