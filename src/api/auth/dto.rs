#[derive(Debug, serde::Deserialize)]
pub struct AuthForm {
	pub username: String,
	pub password: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginQuery {
	pub tab: Option<String>,
}