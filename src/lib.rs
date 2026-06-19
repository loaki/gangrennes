pub mod config;
pub mod controllers;
pub mod db;
pub mod models;
pub mod routes;
pub mod utils;

use axum::{Router, http::HeaderValue};
use sqlx::SqlitePool;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    config::AppConfig,
    db::sqlite::{create_pool, run_migrations},
    utils::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
}

pub async fn build_app(config: &AppConfig) -> AppResult<Router> {
    let pool = create_pool(&config.database_url, config.max_db_connections).await?;
    run_migrations(&pool).await?;

    let state = AppState {
        pool,
        jwt_secret: config.jwt_secret.clone(),
        jwt_expiration_minutes: config.jwt_expiration_minutes,
    };
    let cors = cors_layer(&config.allowed_origin)?;

    Ok(routes::router()
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors))
}

fn cors_layer(allowed_origin: &str) -> AppResult<CorsLayer> {
    if allowed_origin == "*" {
        return Ok(CorsLayer::new().allow_origin(Any));
    }

    let header_value: HeaderValue = allowed_origin
        .parse()
        .map_err(|_| AppError::Config("ALLOWED_ORIGIN is not a valid header value".to_owned()))?;

    Ok(CorsLayer::new().allow_origin(header_value))
}