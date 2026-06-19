pub mod auth;
pub mod docs;
pub mod health;
pub mod items;

use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(docs::routes())
        .merge(health::routes())
        .nest("/auth", auth::routes())
        .nest("/api", items::routes())
}