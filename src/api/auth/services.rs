use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::time::Duration;
use time::OffsetDateTime;

pub const SESSION_COOKIE: &str = "lg_session";

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("username must be 3-32 characters and use letters, numbers, underscores, or hyphens")]
    InvalidUsername,
    #[error("password must be at least 8 characters")]
    InvalidPassword,
    #[error("username already exists")]
    UsernameTaken,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("password hash error")]
    PasswordHash,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

pub async fn register_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<AuthUser, AuthError> {
    validate_username(username)?;
    validate_password(password)?;

    let password_hash = hash_password(password)?;
    let now = now_unix();

    let result = sqlx::query(
        r#"
        INSERT INTO users (username, password_hash, created_at, modification_date)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await;

    let insert = match result {
        Ok(result) => result,
        Err(sqlx::Error::Database(database_error))
            if database_error.message().contains("UNIQUE constraint failed") =>
        {
            return Err(AuthError::UsernameTaken);
        }
        Err(error) => {
            return Err(AuthError::Database(error));
        }
    };

    Ok(AuthUser {
        id: insert.last_insert_rowid(),
        username: username.to_string(),
    })
}

pub async fn authenticate_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<AuthUser, AuthError> {
    let row = sqlx::query(
        r#"
        SELECT id, username, password_hash
        FROM users
        WHERE username = ? COLLATE NOCASE
        LIMIT 1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Err(AuthError::InvalidCredentials);
    };

    let password_hash = row.try_get::<String, _>("password_hash")?;
    verify_password(&password_hash, password)?;

    Ok(AuthUser {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
    })
}

pub async fn create_session(
    pool: &SqlitePool,
    user_id: i64,
    ttl: Duration,
) -> Result<String, sqlx::Error> {
    let token = generate_session_token();
    let token_hash = hash_token(&token);
    let now = now_unix();
    let expires_at = (OffsetDateTime::now_utc() + time::Duration::seconds(ttl.as_secs() as i64))
        .unix_timestamp();

    sqlx::query(
        r#"
        INSERT INTO sessions (user_id, token_hash, expires_at, created_at, modification_date)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn resolve_session(pool: &SqlitePool, token: &str) -> Result<Option<AuthUser>, sqlx::Error> {
    let token_hash = hash_token(token);
    let now = now_unix();

    let row = sqlx::query(
        r#"
        SELECT users.id, users.username
        FROM sessions
        JOIN users ON users.id = sessions.user_id
        WHERE sessions.token_hash = ?
          AND sessions.revoked_at IS NULL
          AND sessions.expires_at > ?
        LIMIT 1
        "#,
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| AuthUser {
        id: row.get("id"),
        username: row.get("username"),
    }))
}

pub async fn revoke_session(pool: &SqlitePool, token: &str) -> Result<(), sqlx::Error> {
    let token_hash = hash_token(token);
    let now = now_unix();

    sqlx::query(
        r#"
        UPDATE sessions
        SET revoked_at = ?
        WHERE token_hash = ?
        "#,
    )
    .bind(now)
    .bind(token_hash)
    .execute(pool)
    .await?;

    Ok(())
}

fn validate_username(username: &str) -> Result<(), AuthError> {
    let valid_length = (3..=32).contains(&username.len());
    let valid_chars = username
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'));

    if valid_length && valid_chars {
        Ok(())
    } else {
        Err(AuthError::InvalidUsername)
    }
}

fn validate_password(password: &str) -> Result<(), AuthError> {
    if (8..=128).contains(&password.len()) {
        Ok(())
    } else {
        Err(AuthError::InvalidPassword)
    }
}

fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AuthError::PasswordHash)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password_hash: &str, password: &str) -> Result<(), AuthError> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|_| AuthError::PasswordHash)?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(()),
        Err(argon2::password_hash::Error::Password) => Err(AuthError::InvalidCredentials),
        Err(_) => Err(AuthError::PasswordHash),
    }
}

fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!("{:x}", digest)
}

fn now_unix() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}