use std::path::Path;

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::utils::error::{AppError, AppResult};

pub async fn create_pool(database_url: &str, max_connections: u32) -> AppResult<SqlitePool> {
    ensure_sqlite_parent_dir(database_url)?;

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;

    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

fn ensure_sqlite_parent_dir(database_url: &str) -> AppResult<()> {
    if !database_url.starts_with("sqlite://") {
        return Ok(());
    }

    let db_path = &database_url[9..];
    if db_path == ":memory:" || db_path.starts_with("file:") {
        return Ok(());
    }

    let parent = Path::new(db_path)
        .parent()
        .ok_or_else(|| AppError::Config("DATABASE_URL does not include a parent directory".to_owned()))?;

    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    std::fs::create_dir_all(parent)?;
    Ok(())
}