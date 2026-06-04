pub mod app_config;
pub mod db_config;

pub use app_config::AppConfig;
pub use db_config::DbConfig;

#[derive(Clone)]
pub struct Config {
    pub app: AppConfig,
    pub db: DbConfig,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            app: AppConfig::from_env(),
            db: DbConfig::from_env(),
        }
    }
}