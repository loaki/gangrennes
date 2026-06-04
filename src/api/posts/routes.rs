use axum::{routing::{get, post}, Router};

use super::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/new", get(handlers::new_page).post(handlers::new_create_submit))
        .route("/posts/{post_id}", get(handlers::post_detail_page))
        .route("/posts/{post_id}/pin", post(handlers::pin_post))
        .route("/posts/{post_id}/reaction", post(handlers::react_to_post))
}