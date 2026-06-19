use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    models::item::{CreateItemRequest, Item},
    utils::error::{AppError, AppResult},
};

pub async fn list_items(pool: &SqlitePool) -> AppResult<Vec<Item>> {
    let items = sqlx::query_as::<_, Item>(
        r#"
        SELECT id, name, description, created_at
        FROM items
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(items)
}

pub async fn get_item(pool: &SqlitePool, id: &str) -> AppResult<Option<Item>> {
    let item = sqlx::query_as::<_, Item>(
        r#"
        SELECT id, name, description, created_at
        FROM items
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(item)
}

pub async fn create_item(pool: &SqlitePool, payload: CreateItemRequest) -> AppResult<Item> {
    let id = Uuid::new_v4().to_string();
    let name = payload.sanitized_name();
    let description = payload.sanitized_description();

    sqlx::query(
        r#"
        INSERT INTO items (id, name, description)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .execute(pool)
    .await?;

    get_item(pool, &id)
        .await?
        .ok_or_else(|| AppError::Internal("inserted item could not be reloaded".to_owned()))
}

pub async fn delete_item(pool: &SqlitePool, id: &str) -> AppResult<bool> {
    let result = sqlx::query(
        r#"
        DELETE FROM items
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}