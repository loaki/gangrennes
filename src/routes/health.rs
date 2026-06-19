use axum::{Router, routing::get};

use crate::{AppState, controllers::health_controller::health_check};

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}