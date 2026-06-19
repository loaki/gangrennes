use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub max_db_connections: u32,
    pub allowed_origin: String,
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let host = env_or_default("HOST", "0.0.0.0");
        let port = parse_env_or_default("PORT", 3000_u16)?;
        let database_url = env_or_default("DATABASE_URL", "sqlite://./data/app.db?mode=rwc");
        let max_db_connections = parse_env_or_default("MAX_DB_CONNECTIONS", 5_u32)?;
        let allowed_origin = env_or_default("ALLOWED_ORIGIN", "*");
        let jwt_secret = env_required("JWT_SECRET")?;
        let jwt_expiration_minutes = parse_env_or_default("JWT_EXPIRATION_MINUTES", 60_i64)?;

        if jwt_secret.len() < 32 {
            return Err(AppError::Config(
                "JWT_SECRET must be at least 32 characters".to_owned(),
            ));
        }

        if jwt_expiration_minutes <= 0 {
            return Err(AppError::Config(
                "JWT_EXPIRATION_MINUTES must be a positive integer".to_owned(),
            ));
        }

        Ok(Self {
            host,
            port,
            database_url,
            max_db_connections,
            allowed_origin,
            jwt_secret,
            jwt_expiration_minutes,
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn env_required(key: &str) -> AppResult<String> {
    std::env::var(key).map_err(|_| AppError::Config(format!("missing required environment variable {key}")))
}

fn parse_env_or_default<T>(key: &str, default: T) -> AppResult<T>
where
    T: std::str::FromStr,
{
    match std::env::var(key) {
        Ok(value) => value.parse::<T>().map_err(|_| {
            AppError::Config(format!("environment variable {key} has an invalid value"))
        }),
        Err(_) => Ok(default),
    }
}