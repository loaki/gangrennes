use askama::Template;
use axum::{
	extract::{Query, State},
	response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
	api::{
		auth::handlers::current_user,
		posts,
		views::{CalendarDay, CalendarEvent, CalendarTemplate, IndexTemplate, ProfileTemplate, TabLink},
	},
	error::AppError,
	state::AppState,
};

#[derive(Deserialize, Default)]
pub struct CalendarQuery {
	pub ym: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct IndexQuery {
	pub auth: Option<String>,
	pub err: Option<String>,
}

const APP_NAME: &str = "LA GANGRENNE";

fn truncate_chars(value: &str, max_chars: usize) -> String {
	if value.chars().count() <= max_chars {
		return value.to_string();
	}

	value.chars().take(max_chars).collect()
}

pub async fn index(
	State(_state): State<AppState>,
	Query(q): Query<IndexQuery>,
) -> Result<Response, AppError> {
	let auth_tab = match q.auth.as_deref() {
		Some("register") => "register",
		_ => "login",
	};

	let error_message = q.err.as_deref().map(|code| match code {
		"invalid_credentials" => "Invalid username or password".to_string(),
		"invalid_username" => "Username must be 3-32 characters and use letters, numbers, underscores, or hyphens".to_string(),
		"invalid_password" => "Password must be at least 8 characters".to_string(),
		"username_taken" => "Username already exists".to_string(),
		"registration_failed" => "Registration failed".to_string(),
		_ => "Authentication failed".to_string(),
	});

	let page = IndexTemplate {
		app_name: APP_NAME.to_string(),
		auth_tab: auth_tab.to_string(),
		open_auth_modal: error_message.is_some(),
		error_message,
	};

	Ok(Html(page.render()?).into_response())
}

pub async fn pinned_page(
	State(state): State<AppState>,
	jar: CookieJar,
) -> Result<Response, AppError> {
	posts::handlers::pinned_page(State(state), jar).await
}

pub async fn calendar_page(
	State(state): State<AppState>,
	jar: CookieJar,
	Query(q): Query<CalendarQuery>,
) -> Result<Response, AppError> {
	let Some(user) = current_user(&state, &jar).await? else {
		return Ok(Redirect::to("/login").into_response());
	};

	// Determine which year/month to display
	let (year, month) = parse_or_current_ym(q.ym.as_deref());
	let year_month = format!("{year:04}-{month:02}");

	let raw_posts = posts::services::list_posts_for_month(&state.pool, user.id, &year_month).await?;

	// Build calendar grid (week starts Monday)
	let days = build_calendar_days(year, month, &raw_posts);

	// Navigation months
	let (prev_y, prev_m) = if month == 1 { (year - 1, 12) } else { (year, month - 1) };
	let (next_y, next_m) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };

	let month_names = ["", "January", "February", "March", "April", "May", "June",
		"July", "August", "September", "October", "November", "December"];

	let page = CalendarTemplate {
		app_name: APP_NAME.to_string(),
		username: user.username,
		tabs: protected_tabs("calendar"),
		month_label: format!("{} {year}", month_names[month as usize]),
		prev_ym: format!("{prev_y:04}-{prev_m:02}"),
		next_ym: format!("{next_y:04}-{next_m:02}"),
		days,
	};

	Ok(Html(page.render()?).into_response())
}

/// Parse "YYYY-MM" or fall back to today.
fn parse_or_current_ym(s: Option<&str>) -> (i32, u8) {
	if let Some(s) = s {
		let parts: Vec<&str> = s.splitn(2, '-').collect();
		if parts.len() == 2 {
			if let (Ok(y), Ok(m)) = (parts[0].parse::<i32>(), parts[1].parse::<u8>()) {
				if m >= 1 && m <= 12 {
					return (y, m);
				}
			}
		}
	}
	let now = time::OffsetDateTime::now_utc();
	(now.year(), now.month() as u8)
}

fn days_in_month(year: i32, month: u8) -> u8 {
	match month {
		1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
		4 | 6 | 9 | 11 => 30,
		2 => {
			let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
			if leap { 29 } else { 28 }
		}
		_ => 30,
	}
}

/// ISO weekday Mon=1..Sun=7 → Monday-based 0-index.
fn weekday_of(year: i32, month: u8, day: u8) -> u8 {
	use time::{Date, Month};
	let m = Month::try_from(month).unwrap_or(time::Month::January);
	if let Ok(d) = Date::from_calendar_date(year, m, day) {
		// time::Weekday Mon=0..Sun=6
		d.weekday() as u8
	} else {
		0
	}
}

fn build_calendar_days(
	year: i32,
	month: u8,
	posts: &[posts::services::PostListItem],
) -> Vec<CalendarDay> {
	let total = days_in_month(year, month);
	let first_wd = weekday_of(year, month, 1); // 0=Mon..6=Sun

	let cells = first_wd as usize + total as usize;
	let rows = (cells + 6) / 7;
	let mut days: Vec<CalendarDay> = Vec::with_capacity(rows * 7);

	// Padding before the 1st
	for _ in 0..first_wd {
		days.push(CalendarDay { day: 0, events: vec![] });
	}

	for d in 1..=total {
		let iso_day = format!("{year:04}-{month:02}-{d:02}");
		let events: Vec<CalendarEvent> = posts
			.iter()
			.filter(|p| {
				// Event spans this day if: start_date <= iso_day <= end_date
				// ISO dates are string-sortable, so we can compare directly
				p.start_date <= iso_day && iso_day <= p.end_date
			})
			.map(|p| CalendarEvent {
				id: p.id,
				title: truncate_chars(&p.title, 10),
				reaction: p.reaction.clone().unwrap_or_else(|| "none".to_string()),
				start_date: p.start_date.clone(),
				end_date: p.end_date.clone(),
			})
			.collect();
		days.push(CalendarDay { day: d, events });
	}

	// Trailing padding to fill last row
	while days.len() % 7 != 0 {
		days.push(CalendarDay { day: 0, events: vec![] });
	}

	days
}

pub async fn profile_page(
	State(state): State<AppState>,
	jar: CookieJar,
) -> Result<Response, AppError> {
	let Some(user) = current_user(&state, &jar).await? else {
		return Ok(Redirect::to("/login").into_response());
	};

	let page = ProfileTemplate {
		app_name: APP_NAME.to_string(),
		username: user.username,
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