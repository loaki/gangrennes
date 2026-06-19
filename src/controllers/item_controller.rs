use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    AppState,
    db::item_repository,
    models::item::{CreateItemRequest, Item},
    utils::error::{AppError, AppResult},
};

#[utoipa::path(
    get,
    path = "/api/items",
    tag = "items",
    responses(
        (status = 200, description = "List items", body = [Item])
    )
)]
pub async fn list_items(State(state): State<AppState>) -> AppResult<Json<Vec<Item>>> {
    let items = item_repository::list_items(&state.pool).await?;
    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/api/items/{id}",
    tag = "items",
    params(
        ("id" = String, Path, description = "Item UUID")
    ),
    responses(
        (status = 200, description = "Found item", body = Item),
        (status = 404, description = "Item not found")
    )
)]
pub async fn get_item(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<Item>> {
    let item_id = id.to_string();
    let item = item_repository::get_item(&state.pool, &item_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("item {item_id} was not found")))?;

    Ok(Json(item))
}

#[utoipa::path(
    post,
    path = "/api/items",
    tag = "items",
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created", body = Item),
        (status = 400, description = "Validation error")
    )
)]
pub async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItemRequest>,
) -> AppResult<(StatusCode, Json<Item>)> {
    payload.validate().map_err(AppError::Validation)?;
    let item = item_repository::create_item(&state.pool, payload).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

#[utoipa::path(
    delete,
    path = "/api/items/{id}",
    tag = "items",
    params(
        ("id" = String, Path, description = "Item UUID")
    ),
    responses(
        (status = 204, description = "Item deleted"),
        (status = 404, description = "Item not found")
    )
)]
pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let item_id = id.to_string();
    let deleted = item_repository::delete_item(&state.pool, &item_id).await?;

    if !deleted {
        return Err(AppError::NotFound(format!("item {item_id} was not found")));
    }

    Ok(StatusCode::NO_CONTENT)
}