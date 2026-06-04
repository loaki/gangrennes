use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use time::OffsetDateTime;

use crate::error::AppError;

#[derive(Clone, Copy)]
pub enum PostFilter {
    Reacted,
    NotReacted,
    Futurs,
}

impl PostFilter {
    pub fn from_query(value: Option<&str>) -> Option<Self> {
        match value {
            Some("reacted") => Some(Self::Reacted),
            Some("not-reacted") => Some(Self::NotReacted),
            Some("futurs") => Some(Self::Futurs),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PostSort {
    CreationDate,
    StartDate,
}

impl PostSort {
    pub fn from_query(value: Option<&str>) -> Self {
        match value {
            Some("start_date") => Self::StartDate,
            _ => Self::CreationDate,
        }
    }
}

pub struct CreatePostInput {
    pub title: String,
    pub image: Option<String>,
    pub description: String,
    pub author_id: i64,
    pub author_name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Clone, Copy)]
pub enum ReactionValue {
    Going,
    NotGoing,
}

impl ReactionValue {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "going" => Some(Self::Going),
            "not_going" => Some(Self::NotGoing),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Going => "going",
            Self::NotGoing => "not_going",
        }
    }
}

#[derive(Clone)]
pub struct PostListItem {
    pub id: i64,
    pub pinned: bool,
    pub title: String,
    pub image: String,
    pub description: String,
    pub author_id: i64,
    pub author_name: String,
    pub start_date: String,
    pub end_date: String,
    pub creation_date: i64,
    pub modification_date: i64,
    pub reacted: bool,
    pub reaction: Option<String>,
    pub going: bool,
    pub not_going: bool,
}

pub async fn create_post(pool: &SqlitePool, input: CreatePostInput) -> Result<(), AppError> {
    validate_title(&input.title)?;
    validate_description(&input.description)?;

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let image = match input.image {
        Some(value) => {
            validate_image(&value)?;
            value
        }
        None => String::new(),
    };
    
    let start_date = match input.start_date {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(normalize_date_to_iso(trimmed, "start_date")?)
            }
        }
        None => None,
    };
    
    let end_date = match input.end_date {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(normalize_date_to_iso(trimmed, "end_date")?)
            }
        }
        None => None,
    };

    // Only validate date relationship if both dates are provided
    if let (Some(ref start), Some(ref end)) = (&start_date, &end_date) {
        if end < start {
            return Err(AppError::BadRequest(
                "end_date must be greater than or equal to start_date".to_string(),
            ));
        }
    }

    sqlx::query(
        r#"
        INSERT INTO posts (
            title,
            pinned,
            image,
            description,
            author_id,
            author_name,
            start_date,
            end_date,
            creation_date,
            modification_date
        )
        VALUES (?, 0, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(input.title)
    .bind(image)
    .bind(input.description)
    .bind(input.author_id)
    .bind(input.author_name)
    .bind(start_date)
    .bind(end_date)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_posts(
    pool: &SqlitePool,
    user_id: i64,
    filter: Option<PostFilter>,
    sort: PostSort,
) -> Result<Vec<PostListItem>, AppError> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            p.id,
            p.pinned,
            p.title,
            p.image,
            p.description,
            p.author_id,
            p.author_name,
            p.start_date,
            p.end_date,
            p.creation_date,
            p.modification_date,
            CASE WHEN r.id IS NULL THEN 0 ELSE 1 END AS reacted,
            r.reaction AS reaction
        FROM posts p
        LEFT JOIN reactions r
            ON r.post_id = p.id
            AND r.user_id = 
        "#,
    );

    query.push_bind(user_id);
    query.push(" WHERE ");

    match filter {
        Some(PostFilter::Reacted) => {
            let _ = query.push("r.user_id IS NOT NULL");
        }
        Some(PostFilter::NotReacted) => {
            let _ = query.push("r.user_id IS NULL");
        }
        Some(PostFilter::Futurs) => {
            query.push("COALESCE(p.start_date, '') > ");
            let _ = query.push_bind(today_iso());
        }
        None => {
            let _ = query.push("1 = 1");
        }
    }

    query.push(" ORDER BY ");
    match sort {
        PostSort::CreationDate => query.push("p.creation_date DESC"),
        PostSort::StartDate => query.push("COALESCE(p.start_date, '') DESC"),
    };
    query.push(", p.id DESC LIMIT 400");

    let rows = query.build().fetch_all(pool).await?;

    let mut posts = Vec::with_capacity(rows.len());
    for row in rows {
        let reacted_flag: i64 = row.try_get("reacted")?;
        let reaction: Option<String> = row.try_get("reaction")?;
        let going = reaction.as_deref() == Some("going");
        let not_going = reaction.as_deref() == Some("not_going");

        posts.push(PostListItem {
            id: row.try_get("id")?,
            pinned: row.try_get::<i64, _>("pinned")? == 1,
            title: row.try_get("title")?,
            image: row.try_get("image")?,
            description: row.try_get("description")?,
            author_id: row.try_get("author_id")?,
            author_name: row.try_get("author_name")?,
            start_date: row.try_get("start_date")?,
            end_date: row.try_get("end_date")?,
            creation_date: row.try_get("creation_date")?,
            modification_date: row.try_get("modification_date")?,
            reacted: reacted_flag == 1,
            reaction,
            going,
            not_going,
        });
    }

    Ok(posts)
}

pub async fn list_pinned_posts(pool: &SqlitePool, user_id: i64) -> Result<Vec<PostListItem>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT
            p.id,
            p.pinned,
            p.title,
            p.image,
            p.description,
            p.author_id,
            p.author_name,
            p.start_date,
            p.end_date,
            p.creation_date,
            p.modification_date,
            CASE WHEN r.id IS NULL THEN 0 ELSE 1 END AS reacted,
            r.reaction AS reaction
        FROM posts p
        LEFT JOIN reactions r ON r.post_id = p.id AND r.user_id = ?
        WHERE p.pinned = 1
        ORDER BY p.creation_date DESC, p.id DESC
        LIMIT 400
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut posts = Vec::with_capacity(rows.len());
    for row in rows {
        let reacted_flag: i64 = row.try_get("reacted")?;
        let reaction: Option<String> = row.try_get("reaction")?;
        let going = reaction.as_deref() == Some("going");
        let not_going = reaction.as_deref() == Some("not_going");

        posts.push(PostListItem {
            id: row.try_get("id")?,
            pinned: row.try_get::<i64, _>("pinned")? == 1,
            title: row.try_get("title")?,
            image: row.try_get("image")?,
            description: row.try_get("description")?,
            author_id: row.try_get("author_id")?,
            author_name: row.try_get("author_name")?,
            start_date: row.try_get("start_date")?,
            end_date: row.try_get("end_date")?,
            creation_date: row.try_get("creation_date")?,
            modification_date: row.try_get("modification_date")?,
            reacted: reacted_flag == 1,
            reaction,
            going,
            not_going,
        });
    }

    Ok(posts)
}

pub async fn get_post_by_id(
    pool: &SqlitePool,
    user_id: i64,
    post_id: i64,
) -> Result<Option<PostListItem>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            p.id,
            p.pinned,
            p.title,
            p.image,
            p.description,
            p.author_id,
            p.author_name,
            p.start_date,
            p.end_date,
            p.creation_date,
            p.modification_date,
            CASE WHEN r.id IS NULL THEN 0 ELSE 1 END AS reacted,
            r.reaction AS reaction
        FROM posts p
        LEFT JOIN reactions r ON r.post_id = p.id AND r.user_id = ?
        WHERE p.id = ?
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(post_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let reacted_flag: i64 = row.try_get("reacted")?;
    let reaction: Option<String> = row.try_get("reaction")?;
    let going = reaction.as_deref() == Some("going");
    let not_going = reaction.as_deref() == Some("not_going");

    Ok(Some(PostListItem {
        id: row.try_get("id")?,
        pinned: row.try_get::<i64, _>("pinned")? == 1,
        title: row.try_get("title")?,
        image: row.try_get("image")?,
        description: row.try_get("description")?,
        author_id: row.try_get("author_id")?,
        author_name: row.try_get("author_name")?,
        start_date: row.try_get("start_date")?,
        end_date: row.try_get("end_date")?,
        creation_date: row.try_get("creation_date")?,
        modification_date: row.try_get("modification_date")?,
        reacted: reacted_flag == 1,
        reaction,
        going,
        not_going,
    }))
}

/// Returns all posts whose start_date falls inside the given month.
/// `year_month` must be `"YYYY-MM"`.
pub async fn list_posts_for_month(
    pool: &SqlitePool,
    user_id: i64,
    year_month: &str,
) -> Result<Vec<PostListItem>, AppError> {
    let prefix = format!("{year_month}-");

    let rows = sqlx::query(
        r#"
        SELECT
            p.id,
            p.pinned,
            p.title,
            COALESCE(p.image, '') AS image,
            p.description,
            p.author_id,
            p.author_name,
            COALESCE(p.start_date, '') AS start_date,
            COALESCE(p.end_date, '') AS end_date,
            p.creation_date,
            p.modification_date,
            CASE WHEN r.id IS NULL THEN 0 ELSE 1 END AS reacted,
            r.reaction AS reaction
        FROM posts p
        LEFT JOIN reactions r ON r.post_id = p.id AND r.user_id = ?
        WHERE p.start_date LIKE ? || '%'
        ORDER BY p.start_date ASC, p.id ASC
        "#,
    )
    .bind(user_id)
    .bind(prefix)
    .fetch_all(pool)
    .await?;

    let mut posts = Vec::with_capacity(rows.len());
    for row in rows {
        let reacted_flag: i64 = row.try_get("reacted")?;
        let reaction: Option<String> = row.try_get("reaction")?;
        let going = reaction.as_deref() == Some("going");
        let not_going = reaction.as_deref() == Some("not_going");

        posts.push(PostListItem {
            id: row.try_get("id")?,
            pinned: row.try_get::<i64, _>("pinned")? == 1,
            title: row.try_get("title")?,
            image: row.try_get("image")?,
            description: row.try_get("description")?,
            author_id: row.try_get("author_id")?,
            author_name: row.try_get("author_name")?,
            start_date: row.try_get("start_date")?,
            end_date: row.try_get("end_date")?,
            creation_date: row.try_get("creation_date")?,
            modification_date: row.try_get("modification_date")?,
            reacted: reacted_flag == 1,
            reaction,
            going,
            not_going,
        });
    }

    Ok(posts)
}

pub async fn set_pinned(pool: &SqlitePool, post_id: i64, pinned: bool) -> Result<(), AppError> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query(
        r#"
        UPDATE posts
        SET pinned = ?, modification_date = ?
        WHERE id = ?
        "#,
    )
    .bind(if pinned { 1_i64 } else { 0_i64 })
    .bind(now)
    .bind(post_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_reaction(
    pool: &SqlitePool,
    user_id: i64,
    post_id: i64,
    reaction: ReactionValue,
) -> Result<(), AppError> {
    let current_reaction: Option<String> = sqlx::query_scalar(
        r#"
        SELECT reaction
        FROM reactions
        WHERE user_id = ? AND post_id = ?
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(post_id)
    .fetch_optional(pool)
    .await?;

    if current_reaction.as_deref() == Some(reaction.as_str()) {
        sqlx::query(
            r#"
            DELETE FROM reactions
            WHERE user_id = ? AND post_id = ?
            "#,
        )
        .bind(user_id)
        .bind(post_id)
        .execute(pool)
        .await?;

        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO reactions (user_id, post_id, reaction)
        VALUES (?, ?, ?)
        ON CONFLICT(user_id, post_id)
        DO UPDATE SET reaction = excluded.reaction
        "#,
    )
    .bind(user_id)
    .bind(post_id)
    .bind(reaction.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

fn validate_title(value: &str) -> Result<(), AppError> {
    let value = value.trim();
    if value.is_empty() || value.len() > 160 {
        return Err(AppError::BadRequest(
            "title must be between 1 and 160 characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_description(value: &str) -> Result<(), AppError> {
    let value = value.trim();
    if value.is_empty() || value.len() > 6000 {
        return Err(AppError::BadRequest(
            "description must be between 1 and 6000 characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_image(value: &str) -> Result<(), AppError> {
    if value.len() > 2048 {
        return Err(AppError::BadRequest(
            "image must be at most 2048 characters".to_string(),
        ));
    }

    if value.starts_with("javascript:") {
        return Err(AppError::BadRequest(
            "image cannot use javascript scheme".to_string(),
        ));
    }

    Ok(())
}

fn validate_date_yyyy_mm_dd(value: &str, field_name: &str) -> Result<(), AppError> {
    if value.len() != 10 {
        return Err(AppError::BadRequest(format!(
            "{field_name} must use YYYY-MM-DD format"
        )));
    }

    let bytes = value.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(AppError::BadRequest(format!(
            "{field_name} must use YYYY-MM-DD format"
        )));
    }

    let year = parse_number(&value[0..4], field_name)?;
    let month = parse_number(&value[5..7], field_name)?;
    let day = parse_number(&value[8..10], field_name)?;

    if !(1970..=2100).contains(&year) {
        return Err(AppError::BadRequest(format!(
            "{field_name} year must be between 1970 and 2100"
        )));
    }

    if !(1..=12).contains(&month) {
        return Err(AppError::BadRequest(format!(
            "{field_name} month must be between 1 and 12"
        )));
    }

    let max_day = days_in_month(year, month);
    if day < 1 || day > max_day {
        return Err(AppError::BadRequest(format!(
            "{field_name} day is invalid for this month"
        )));
    }

    Ok(())
}

fn normalize_date_to_iso(value: &str, field_name: &str) -> Result<String, AppError> {
    let value = value.trim();

    if value.len() == 10 && value.as_bytes()[4] == b'-' && value.as_bytes()[7] == b'-' {
        validate_date_yyyy_mm_dd(value, field_name)?;
        return Ok(value.to_string());
    }

    if value.len() == 10 && value.as_bytes()[2] == b'/' && value.as_bytes()[5] == b'/' {
        let day = &value[0..2];
        let month = &value[3..5];
        let year = &value[6..10];

        if !day.chars().all(|c| c.is_ascii_digit())
            || !month.chars().all(|c| c.is_ascii_digit())
            || !year.chars().all(|c| c.is_ascii_digit())
        {
            return Err(AppError::BadRequest(format!(
                "{field_name} must use DD/MM/YYYY or YYYY-MM-DD format"
            )));
        }

        let iso = format!("{year}-{month}-{day}");
        validate_date_yyyy_mm_dd(&iso, field_name)?;
        return Ok(iso);
    }

    Err(AppError::BadRequest(format!(
        "{field_name} must use DD/MM/YYYY or YYYY-MM-DD format"
    )))
}

fn parse_number(input: &str, field_name: &str) -> Result<i32, AppError> {
    if !input.chars().all(|character| character.is_ascii_digit()) {
        return Err(AppError::BadRequest(format!(
            "{field_name} must use YYYY-MM-DD format"
        )));
    }

    input
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest(format!("{field_name} must use YYYY-MM-DD format")))
}

fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn today_iso() -> String {
    let today = OffsetDateTime::now_utc().date();
    format!(
        "{:04}-{:02}-{:02}",
        today.year(),
        u8::from(today.month()),
        today.day()
    )
}