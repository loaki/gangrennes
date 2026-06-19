use axum::{Router, routing::post};

use crate::{
    AppState,
    controllers::auth_controller::{login, register},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}