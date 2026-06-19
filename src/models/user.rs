use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: String,
    pub name: String,
    pub creation_date: String,
    pub modification_date: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserWithPassword {
    pub id: String,
    pub name: String,
    pub password_hash: String,
    pub creation_date: String,
    pub modification_date: String,
}

impl From<UserWithPassword> for User {
    fn from(value: UserWithPassword) -> Self {
        Self {
            id: value.id,
            name: value.name,
            creation_date: value.creation_date,
            modification_date: value.modification_date,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_user_name(&self.name)?;
        validate_password(&self.password)?;
        Ok(())
    }

    pub fn sanitized_name(&self) -> String {
        self.name.trim().to_owned()
    }
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), String> {
        validate_user_name(&self.name)?;
        if self.password.is_empty() {
            return Err("password is required".to_owned());
        }
        Ok(())
    }

    pub fn sanitized_name(&self) -> String {
        self.name.trim().to_owned()
    }
}

fn validate_user_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("name is required".to_owned());
    }

    if trimmed.len() > 80 {
        return Err("name must be 80 characters or less".to_owned());
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 12 {
        return Err("password must be at least 12 characters".to_owned());
    }

    if password.len() > 256 {
        return Err("password must be 256 characters or less".to_owned());
    }

    Ok(())
}