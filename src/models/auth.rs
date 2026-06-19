use serde::Serialize;
use utoipa::ToSchema;

use crate::models::user::User;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}