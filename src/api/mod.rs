pub mod auth;
pub mod pages;
pub mod posts;
pub mod views;

use axum::{extract::DefaultBodyLimit, Router};
use tower_http::{
    compression::CompressionLayer,
    services::ServeDir,
    trace::TraceLayer,
};

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(auth::routes::router())
        .merge(pages::routes::router())
        .merge(posts::routes::router())
        .nest_service("/uploads", ServeDir::new("uploads"))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
