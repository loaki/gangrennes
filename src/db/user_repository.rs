use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    models::user::{User, UserWithPassword},
    utils::error::{AppError, AppResult},
};

pub async fn create_user(pool: &SqlitePool, name: &str, password_hash: &str) -> AppResult<User> {
    let id = Uuid::new_v4().to_string();

    let insert_result = sqlx::query(
        r#"
        INSERT INTO users (id, name, password_hash)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(password_hash)
    .execute(pool)
    .await;

    if let Err(error) = insert_result {
        if is_unique_name_violation(&error) {
            return Err(AppError::Conflict("user name already exists".to_owned()));
        }
        return Err(AppError::Database(error));
    }

    get_user_by_id(pool, &id)
        .await?
        .ok_or_else(|| AppError::Internal("created user could not be reloaded".to_owned()))
}

pub async fn get_user_by_name(pool: &SqlitePool, name: &str) -> AppResult<Option<UserWithPassword>> {
    let user = sqlx::query_as::<_, UserWithPassword>(
        r#"
        SELECT id, name, password_hash, creation_date, modification_date
        FROM users
        WHERE name = ?1
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

async fn get_user_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, creation_date, modification_date
        FROM users
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

fn is_unique_name_violation(error: &sqlx::Error) -> bool {
    let Some(database_error) = error.as_database_error() else {
        return false;
    };

    // SQLite extended code 2067 = SQLITE_CONSTRAINT_UNIQUE
    if database_error.code().as_deref() == Some("2067") {
        return true;
    }

    database_error
        .message()
        .contains("UNIQUE constraint failed: users.name")
}