use axum::{Router, routing::get};

use crate::{
    AppState,
    controllers::item_controller::{create_item, delete_item, get_item, list_items},
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/items", get(list_items).post(create_item)).route(
        "/items/{id}",
        get(get_item).delete(delete_item),
    )
}