use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReactionForm {
	pub reaction: String,
	pub filter: Option<String>,
	pub sort: Option<String>,
	pub tab: Option<String>,
}

#[derive(Deserialize)]
pub struct PinForm {
	pub pinned: String,
	pub filter: Option<String>,
	pub sort: Option<String>,
	pub tab: Option<String>,
}

#[derive(Deserialize)]
pub struct NewQuery {
	pub filter: Option<String>,
	pub sort: Option<String>,
}

#[derive(Deserialize)]
pub struct CreatePostForm {
	pub title: String,
	pub image: Option<String>,
	pub description: String,
	pub start_date: Option<String>,
	pub end_date: Option<String>,
	pub filter: Option<String>,
	pub sort: Option<String>,
}