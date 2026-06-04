use axum::{routing::get, Router};

use super::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::index))
        .route("/pinned", get(handlers::pinned_page))
        .route("/calendar", get(handlers::calendar_page))
        .route("/profile", get(handlers::profile_page))
}