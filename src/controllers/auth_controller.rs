use axum::{
    Json,
    extract::State,
    http::StatusCode,
};

use crate::{
    AppState,
    db::user_repository,
    models::{
        auth::AuthResponse,
        user::{LoginRequest, RegisterRequest, User},
    },
    utils::{
        auth::{hash_password, issue_jwt, verify_password},
        error::{AppError, AppResult},
    },
};

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "User already exists")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    payload.validate().map_err(AppError::Validation)?;

    let name = payload.sanitized_name();
    let password_hash = hash_password(&payload.password)?;
    let user = user_repository::create_user(&state.pool, &name, &password_hash).await?;

    let token = issue_token(&state, &user)?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse { token, user }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login succeeded", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    payload.validate().map_err(AppError::Validation)?;

    let name = payload.sanitized_name();
    let user_with_password = match user_repository::get_user_by_name(&state.pool, &name).await? {
        Some(user) => user,
        None => {
            // Keep a similar CPU profile to reduce user-enumeration timing side-channel signal.
            let _ = hash_password(&payload.password);
            return Err(AppError::Unauthorized("invalid credentials".to_owned()));
        }
    };

    let password_is_valid = verify_password(&payload.password, &user_with_password.password_hash)?;

    if !password_is_valid {
        return Err(AppError::Unauthorized("invalid credentials".to_owned()));
    }

    let user: User = user_with_password.into();
    let token = issue_token(&state, &user)?;

    Ok(Json(AuthResponse { token, user }))
}

fn issue_token(state: &AppState, user: &User) -> AppResult<String> {
    issue_jwt(
        &user.id,
        &user.name,
        &state.jwt_secret,
        state.jwt_expiration_minutes,
    )
}