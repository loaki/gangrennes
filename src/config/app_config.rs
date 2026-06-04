use std::{env, net::SocketAddr, time::Duration};

#[derive(Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub cookie_secure: bool,
    pub session_ttl: Duration,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let bind_addr = env::var("BIND_ADDR")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 3000)));

        let cookie_secure = env::var("COOKIE_SECURE")
            .ok()
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false);

        let session_ttl_seconds = env::var("SESSION_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(86_400);

        Self {
            bind_addr,
            cookie_secure,
            session_ttl: Duration::from_secs(session_ttl_seconds),
        }
    }
}