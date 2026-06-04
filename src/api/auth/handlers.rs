use axum::{
	extract::State,
	response::{IntoResponse, Redirect, Response},
	Form,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;

use crate::{
	api::{
		auth::{dto::AuthForm, services::{self as auth, AuthError}},
	},
	error::AppError,
	state::AppState,
};

pub async fn login_redirect_home() -> Response {
	Redirect::to("/").into_response()
}

pub async fn login_submit(
	State(state): State<AppState>,
	jar: CookieJar,
	Form(form): Form<AuthForm>,
) -> Result<Response, AppError> {
	let username = form.username.trim();
	let password = form.password;

	match auth::authenticate_user(&state.pool, username, &password).await {
		Ok(user) => {
			let session_token =
				auth::create_session(&state.pool, user.id, state.config.app.session_ttl).await?;
			let jar = jar.add(build_session_cookie(&state, session_token));
			Ok((jar, Redirect::to("/new")).into_response())
		}
		Err(AuthError::InvalidCredentials) => redirect_home_auth_error("login", "invalid_credentials"),
		Err(AuthError::Database(error)) => Err(AppError::Database(error)),
		Err(AuthError::PasswordHash) => Err(AppError::Internal),
		Err(AuthError::InvalidUsername | AuthError::InvalidPassword | AuthError::UsernameTaken) => {
			redirect_home_auth_error("login", "invalid_credentials")
		}
	}
}

pub async fn register_submit(
	State(state): State<AppState>,
	jar: CookieJar,
	Form(form): Form<AuthForm>,
) -> Result<Response, AppError> {
	let username = form.username.trim();
	let password = form.password;

	match auth::register_user(&state.pool, username, &password).await {
		Ok(user) => {
			let session_token =
				auth::create_session(&state.pool, user.id, state.config.app.session_ttl).await?;
			let jar = jar.add(build_session_cookie(&state, session_token));
			Ok((jar, Redirect::to("/new")).into_response())
		}
		Err(AuthError::InvalidUsername) => redirect_home_auth_error("register", "invalid_username"),
		Err(AuthError::InvalidPassword) => {
			redirect_home_auth_error("register", "invalid_password")
		}
		Err(AuthError::UsernameTaken) => redirect_home_auth_error("register", "username_taken"),
		Err(AuthError::Database(error)) => Err(AppError::Database(error)),
		Err(AuthError::PasswordHash) => Err(AppError::Internal),
		Err(AuthError::InvalidCredentials) => redirect_home_auth_error("register", "registration_failed"),
	}
}

pub async fn logout_submit(
	State(state): State<AppState>,
	jar: CookieJar,
) -> Result<Response, AppError> {
	if let Some(cookie) = jar.get(auth::SESSION_COOKIE) {
		auth::revoke_session(&state.pool, cookie.value()).await?;
	}

	let mut cookie = Cookie::new(auth::SESSION_COOKIE, "");
	cookie.set_path("/");
	cookie.set_http_only(true);
	cookie.set_same_site(SameSite::Lax);
	cookie.set_secure(state.config.app.cookie_secure);
	cookie.make_removal();

	Ok((jar.remove(cookie), Redirect::to("/")).into_response())
}

pub async fn current_user(
	state: &AppState,
	jar: &CookieJar,
) -> Result<Option<auth::AuthUser>, AppError> {
	let Some(cookie) = jar.get(auth::SESSION_COOKIE) else {
		return Ok(None);
	};

	Ok(auth::resolve_session(&state.pool, cookie.value()).await?)
}

fn build_session_cookie(state: &AppState, session_token: String) -> Cookie<'static> {
	let mut cookie = Cookie::new(auth::SESSION_COOKIE, session_token);
	cookie.set_path("/");
	cookie.set_http_only(true);
	cookie.set_same_site(SameSite::Lax);
	cookie.set_secure(state.config.app.cookie_secure);
	cookie
}

fn redirect_home_auth_error(tab: &str, err: &str) -> Result<Response, AppError> {
	let location = format!("/?auth={tab}&err={err}");
	Ok(Redirect::to(&location).into_response())
}