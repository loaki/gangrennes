use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    name: String,
    iat: usize,
    exp: usize,
}

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AppError::Internal("failed to hash password".to_owned()))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| AppError::Internal("stored password hash is invalid".to_owned()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn issue_jwt(
    user_id: &str,
    user_name: &str,
    jwt_secret: &str,
    expiration_minutes: i64,
) -> AppResult<String> {
    let now = Utc::now().timestamp();
    let exp = now
        .checked_add(expiration_minutes.saturating_mul(60))
        .ok_or_else(|| AppError::Internal("invalid jwt expiration configuration".to_owned()))?;

    let claims = Claims {
        sub: user_id.to_owned(),
        name: user_name.to_owned(),
        iat: now as usize,
        exp: exp as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal("failed to generate jwt".to_owned()))
}