use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::{path::Path, str::FromStr, time::Duration};

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    ensure_parent_directory(database_url);

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(8)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await
}

fn ensure_parent_directory(database_url: &str) {
    if let Some(path) = database_path(database_url) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
}

fn database_path(database_url: &str) -> Option<&Path> {
    let stripped = database_url.strip_prefix("sqlite://")?;
    let path = Path::new(stripped);

    if path.as_os_str().is_empty() {
        None
    } else {
        Some(path)
    }
}