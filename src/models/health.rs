use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok",
            service: "gangrennes-api",
        }
    }
}