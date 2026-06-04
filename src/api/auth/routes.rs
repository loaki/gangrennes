use axum::{routing::{get, post}, Router};

use super::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
    .route("/login", get(handlers::login_redirect_home).post(handlers::login_submit))
        .route("/register", post(handlers::register_submit))
        .route("/logout", post(handlers::logout_submit))
}