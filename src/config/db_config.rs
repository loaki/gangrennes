use std::env;

#[derive(Clone)]
pub struct DbConfig {
    pub database_url: String,
}

impl DbConfig {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://data/gangrennes.sqlite3".to_string());

        Self { database_url }
    }
}