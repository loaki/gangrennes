use crate::config::Config;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: SqlitePool,
}

impl AppState {
    pub fn new(config: Config, pool: SqlitePool) -> Self {
        Self { config, pool }
    }
}
