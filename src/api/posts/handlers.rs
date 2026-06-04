use askama::Template;
use axum::{
	extract::{Multipart, Path, Query, State},
	http::{HeaderMap, StatusCode},
	response::{Html, IntoResponse, Redirect, Response},
	Form,
};
use axum_extra::extract::CookieJar;
use rand::RngCore;

use crate::{
	api::{
		auth::handlers::current_user,
		posts::{
			dto::{NewQuery, PinForm, ReactionForm},
			services::{self as posts, CreatePostInput, PostFilter, PostSort, ReactionValue},
		},
		views::{NewTemplate, PostDetailTemplate, PostItemTemplate, SelectOption, TabLink},
	},
	error::AppError,
	state::AppState,
};

const APP_NAME: &str = "LA GANGRENNE";

fn truncate_chars(value: &str, max_chars: usize) -> String {
	if value.chars().count() <= max_chars {
		return value.to_string();
	}

	value.chars().take(max_chars).collect()
}

pub async fn new_page(
	State(state): State<AppState>,
	jar: CookieJar,
	Query(query): Query<NewQuery>,
) -> Result<Response, AppError> {
	let Some(user) = current_user(&state, &jar).await? else {
		return Ok(Redirect::to("/login").into_response());
	};

	render_new_page(&state, &user, query.filter.as_deref(), query.sort.as_deref(), None).await
}

pub async fn pinned_page(State(state): State<AppState>, jar: CookieJar) -> Result<Response, AppError> {
	let Some(user) = current_user(&state, &jar).await? else {
		return Ok(Redirect::to("/login").into_response());
	};

	render_pinned_page(&state, &user, None).await
}

pub async fn post_detail_page(
	State(state): State<AppState>,
	Path(post_id): Path<i64>,
	jar: CookieJar,
) -> Result<Response, AppError> {
	let Some(user) = current_user(&state, &jar).await? else {
		return Ok(Redirect::to("/").into_response());
	};

	let Some(post) = posts::get_post_by_id(&state.pool, user.id, post_id).await? else {
		return Err(AppError::BadRequest("post not found".to_string()));
	};

	let page = PostDetailTemplate {
		app_name: APP_NAME.to_string(),
		username: user.username,
		creation_date: post.creation_date,
		modification_date: post.modification_date,
		post: PostItemTemplate {
			id: post.id,
			pinned: post.pinned,
			title: post.title,
			image: post.image,
			description: post.description,
			author_name: post.author_name,
			start_date: post.start_date,
			end_date: post.end_date,
			reacted: post.reacted,
			going: post.going,
			not_going: post.not_going,
		},
	};

	Ok(Html(page.render()?).into_response())
}

pub async fn new_create_submit(
	State(state): State<AppState>,
	jar: CookieJar,
	mut multipart: Multipart,
) -> Result<Response, AppError> {
	let Some(user) = current_user(&state, &jar).await? else {
		return Ok(Redirect::to("/login").into_response());
	};

	let form = parse_create_post_multipart(&mut multipart).await?;

	let selected_filter = form.filter.clone();
	let selected_sort = form.sort.clone();

	let input = CreatePostInput {
		title: form.title.trim().to_string(),
		image: optional_trimmed(form.image),
		description: form.description.trim().to_string(),
		author_id: user.id,
		author_name: user.username.clone(),
		start_date: optional_trimmed(form.start_date),
		end_date: optional_trimmed(form.end_date),
	};

	match posts::create_post(&state.pool, input).await {
		Ok(()) => {
			let target = format!(
				"/new?filter={}&sort={}",
				selected_filter.as_deref().unwrap_or("all"),
				selected_sort.as_deref().unwrap_or("creation_date")
			);
			Ok(Redirect::to(&target).into_response())
		}
		Err(AppError::BadRequest(message)) => {
			render_new_page(
				&state,
				&user,
				selected_filter.as_deref(),
				selected_sort.as_deref(),
				Some(message),
			)
			.await
		}
		Err(error) => Err(error),
	}
}

struct ParsedCreatePost {
	title: String,
	image: Option<String>,
	description: String,
	start_date: Option<String>,
	end_date: Option<String>,
	filter: Option<String>,
	sort: Option<String>,
}

const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;

async fn parse_create_post_multipart(multipart: &mut Multipart) -> Result<ParsedCreatePost, AppError> {
	let mut title: Option<String> = None;
	let mut image: Option<String> = None;
	let mut description: Option<String> = None;
	let mut start_date: Option<String> = None;
	let mut end_date: Option<String> = None;
	let mut filter: Option<String> = None;
	let mut sort: Option<String> = None;

	while let Some(field) = multipart
		.next_field()
		.await
		.map_err(|_| AppError::BadRequest("invalid multipart payload".to_string()))?
	{
		let Some(name) = field.name() else {
			continue;
		};

		match name {
			"title" => {
				title = Some(
					field
						.text()
						.await
						.map_err(|_| AppError::BadRequest("invalid title field".to_string()))?,
				);
			}
			"description" => {
				description = Some(
					field
						.text()
						.await
						.map_err(|_| AppError::BadRequest("invalid description field".to_string()))?,
				);
			}
			"start_date" => {
				start_date = Some(
					field
						.text()
						.await
						.map_err(|_| AppError::BadRequest("invalid start_date field".to_string()))?,
				);
			}
			"end_date" => {
				end_date = Some(
					field
						.text()
						.await
						.map_err(|_| AppError::BadRequest("invalid end_date field".to_string()))?,
				);
			}
			"filter" => {
				filter = Some(
					field
						.text()
						.await
						.map_err(|_| AppError::BadRequest("invalid filter field".to_string()))?,
				);
			}
			"sort" => {
				sort = Some(
					field
						.text()
						.await
						.map_err(|_| AppError::BadRequest("invalid sort field".to_string()))?,
				);
			}
			"image" => {
				let content_type = field.content_type().unwrap_or_default().to_string();

				let bytes = field
					.bytes()
					.await
					.map_err(|_| AppError::BadRequest("invalid image upload".to_string()))?;

				if bytes.is_empty() {
					continue;
				}

				if !content_type.is_empty() && !is_allowed_image_content_type(&content_type) {
					return Err(AppError::BadRequest("unsupported image content type".to_string()));
				}

				if bytes.len() > MAX_IMAGE_BYTES {
					return Err(AppError::BadRequest("image must be at most 5MB".to_string()));
				}

				let Some(extension) = detect_image_extension(&bytes) else {
					return Err(AppError::BadRequest(
						"unsupported image format; allowed: jpg, png, webp, gif".to_string(),
					));
				};

				image = Some(save_uploaded_image(&bytes, extension).await?);
			}
			_ => {}
		}
	}

	let title = title.ok_or_else(|| AppError::BadRequest("missing title".to_string()))?;
	let description =
		description.ok_or_else(|| AppError::BadRequest("missing description".to_string()))?;

	Ok(ParsedCreatePost {
		title,
		image,
		description,
		start_date,
		end_date,
		filter,
		sort,
	})
}

fn is_allowed_image_content_type(content_type: &str) -> bool {
	matches!(
		content_type,
		"image/jpeg" | "image/png" | "image/webp" | "image/gif"
	)
}

fn detect_image_extension(bytes: &[u8]) -> Option<&'static str> {
	const PNG_SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

	if bytes.starts_with(&PNG_SIG) {
		return Some("png");
	}

	if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
		return Some("jpg");
	}

	if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
		return Some("gif");
	}

	if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
		return Some("webp");
	}

	None
}

async fn save_uploaded_image(bytes: &[u8], extension: &str) -> Result<String, AppError> {
	let mut random_bytes = [0_u8; 16];
	rand::thread_rng().fill_bytes(&mut random_bytes);

	let mut filename = String::with_capacity(32);
	for value in random_bytes {
		filename.push_str(&format!("{value:02x}"));
	}

	tokio::fs::create_dir_all("uploads")
		.await
		.map_err(|_| AppError::Internal)?;

	let relative_path = format!("uploads/{filename}.{extension}");
	tokio::fs::write(&relative_path, bytes)
		.await
		.map_err(|_| AppError::Internal)?;

	Ok(format!("/uploads/{filename}.{extension}"))
}

pub async fn react_to_post(
	State(state): State<AppState>,
	Path(post_id): Path<i64>,
	headers: HeaderMap,
	jar: CookieJar,
	Form(form): Form<ReactionForm>,
) -> Result<Response, AppError> {
	let Some(user) = current_user(&state, &jar).await? else {
		if is_ajax_request(&headers) {
			return Ok(StatusCode::UNAUTHORIZED.into_response());
		}
		return Ok(Redirect::to("/login").into_response());
	};

	let Some(reaction) = ReactionValue::from_str(form.reaction.trim()) else {
		return Err(AppError::BadRequest("invalid reaction".to_string()));
	};

	posts::set_reaction(&state.pool, user.id, post_id, reaction).await?;

	if is_ajax_request(&headers) {
		return Ok(StatusCode::NO_CONTENT.into_response());
	}

	let target = target_from_form(form.tab.as_deref(), form.filter.as_deref(), form.sort.as_deref());

	Ok(Redirect::to(&target).into_response())
}

pub async fn pin_post(
	State(state): State<AppState>,
	Path(post_id): Path<i64>,
	headers: HeaderMap,
	jar: CookieJar,
	Form(form): Form<PinForm>,
) -> Result<Response, AppError> {
	let Some(_user) = current_user(&state, &jar).await? else {
		if is_ajax_request(&headers) {
			return Ok(StatusCode::UNAUTHORIZED.into_response());
		}
		return Ok(Redirect::to("/login").into_response());
	};

	let pinned = match form.pinned.as_str() {
		"1" => true,
		"0" => false,
		_ => return Err(AppError::BadRequest("invalid pinned value".to_string())),
	};

	posts::set_pinned(&state.pool, post_id, pinned).await?;

	if is_ajax_request(&headers) {
		return Ok(StatusCode::NO_CONTENT.into_response());
	}

	let target = target_from_form(form.tab.as_deref(), form.filter.as_deref(), form.sort.as_deref());
	Ok(Redirect::to(&target).into_response())
}

fn is_ajax_request(headers: &HeaderMap) -> bool {
	headers
		.get("x-requested-with")
		.and_then(|v| v.to_str().ok())
		.map(|v| v.eq_ignore_ascii_case("xmlhttprequest"))
		.unwrap_or(false)
}

async fn render_new_page(
	state: &AppState,
	user: &crate::api::auth::services::AuthUser,
	filter_query: Option<&str>,
	sort_query: Option<&str>,
	error_message: Option<String>,
) -> Result<Response, AppError> {
	let selected_filter = normalize_filter(filter_query);
	let selected_sort = normalize_sort(sort_query);

	let posts = posts::list_posts(
		&state.pool,
		user.id,
		PostFilter::from_query(Some(selected_filter)),
		PostSort::from_query(Some(selected_sort)),
	)
	.await?;

	let page = NewTemplate {
		app_name: APP_NAME.to_string(),
		current_tab: "new".to_string(),
		show_create_controls: true,
		username: user.username.clone(),
		tabs: protected_tabs("new"),
		filter_options: filter_options(selected_filter),
		sort_options: sort_options(selected_sort),
		selected_filter: selected_filter.to_string(),
		selected_sort: selected_sort.to_string(),
		error_message,
		posts: posts
			.into_iter()
			.map(|post| PostItemTemplate {
				id: post.id,
				pinned: post.pinned,
				title: truncate_chars(&post.title, 20),
				image: post.image,
				description: post.description,
				author_name: post.author_name,
				start_date: post.start_date,
				end_date: post.end_date,
				reacted: post.reacted,
				going: post.going,
				not_going: post.not_going,
			})
			.collect(),
	};

	Ok(Html(page.render()?).into_response())
}

async fn render_pinned_page(
	state: &AppState,
	user: &crate::api::auth::services::AuthUser,
	error_message: Option<String>,
) -> Result<Response, AppError> {
	let posts = posts::list_pinned_posts(&state.pool, user.id).await?;

	let page = NewTemplate {
		app_name: APP_NAME.to_string(),
		current_tab: "pinned".to_string(),
		show_create_controls: false,
		username: user.username.clone(),
		tabs: protected_tabs("pinned"),
		filter_options: filter_options("all"),
		sort_options: sort_options("creation_date"),
		selected_filter: "all".to_string(),
		selected_sort: "creation_date".to_string(),
		error_message,
		posts: posts
			.into_iter()
			.map(|post| PostItemTemplate {
				id: post.id,
				pinned: post.pinned,
				title: truncate_chars(&post.title, 20),
				image: post.image,
				description: post.description,
				author_name: post.author_name,
				start_date: post.start_date,
				end_date: post.end_date,
				reacted: post.reacted,
				going: post.going,
				not_going: post.not_going,
			})
			.collect(),
	};

	Ok(Html(page.render()?).into_response())
}

fn protected_tabs(active: &str) -> Vec<TabLink> {
	[("/pinned", "pinned"), ("/new", "new"), ("/calendar", "calendar")]
		.into_iter()
		.map(|(href, label)| TabLink {
			href: href.to_string(),
			label: label.to_string(),
			active: label == active,
		})
		.collect()
}

fn normalize_filter(value: Option<&str>) -> &'static str {
	match value {
		Some("reacted") => "reacted",
		Some("not-reacted") => "not-reacted",
		Some("futurs") => "futurs",
		_ => "all",
	}
}

fn normalize_sort(value: Option<&str>) -> &'static str {
	match value {
		Some("start_date") => "start_date",
		_ => "creation_date",
	}
}

fn filter_options(selected_filter: &str) -> Vec<SelectOption> {
	[
		("all", "all"),
		("reacted", "reacted"),
		("not-reacted", "not reacted"),
		("futurs", "futurs"),
	]
	.into_iter()
	.map(|(value, label)| SelectOption {
		value: value.to_string(),
		label: label.to_string(),
		selected: selected_filter == value,
	})
	.collect()
}

fn sort_options(selected_sort: &str) -> Vec<SelectOption> {
	[("creation_date", "creation_date"), ("start_date", "start_date")]
		.into_iter()
		.map(|(value, label)| SelectOption {
			value: value.to_string(),
			label: label.to_string(),
			selected: selected_sort == value,
		})
		.collect()
}

fn optional_trimmed(value: Option<String>) -> Option<String> {
	value.and_then(|v| {
		let trimmed = v.trim().to_string();
		if trimmed.is_empty() {
			None
		} else {
			Some(trimmed)
		}
	})
}

fn target_from_form(tab: Option<&str>, filter: Option<&str>, sort: Option<&str>) -> String {
	match tab {
		Some("pinned") => "/pinned".to_string(),
		_ => format!(
			"/new?filter={}&sort={}",
			filter.unwrap_or("all"),
			sort.unwrap_or("creation_date")
		),
	}
}