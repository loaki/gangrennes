use askama::Template;

#[derive(Clone)]
pub struct TabLink {
    pub href: String,
    pub label: String,
    pub active: bool,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub app_name: String,
    pub auth_tab: String,
    pub open_auth_modal: bool,
    pub error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub app_name: String,
    pub active_tab: String,
    pub error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "protected.html")]
pub struct ProtectedTemplate {
    pub app_name: String,
    pub username: String,
    pub page_title: String,
    pub tabs: Vec<TabLink>,
}

#[derive(Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Clone)]
pub struct PostItemTemplate {
    pub id: i64,
    pub pinned: bool,
    pub title: String,
    pub image: String,
    pub description: String,
    pub author_name: String,
    pub start_date: String,
    pub end_date: String,
    pub reacted: bool,
    pub going: bool,
    pub not_going: bool,
}

#[derive(Template)]
#[template(path = "new.html")]
pub struct NewTemplate {
    pub app_name: String,
    pub current_tab: String,
    pub show_create_controls: bool,
    pub username: String,
    pub tabs: Vec<TabLink>,
    pub filter_options: Vec<SelectOption>,
    pub sort_options: Vec<SelectOption>,
    pub selected_filter: String,
    pub selected_sort: String,
    pub error_message: Option<String>,
    pub posts: Vec<PostItemTemplate>,
}

#[derive(Template)]
#[template(path = "post_detail.html")]
pub struct PostDetailTemplate {
    pub app_name: String,
    pub username: String,
    pub post: PostItemTemplate,
    pub creation_date: i64,
    pub modification_date: i64,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub app_name: String,
    pub username: String,
}

#[derive(Clone)]
pub struct CalendarEvent {
    pub id: i64,
    pub title: String,
    /// "none" | "going" | "not_going"
    pub reaction: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Clone)]
pub struct CalendarDay {
    /// day-of-month number, 0 = padding cell
    pub day: u8,
    pub events: Vec<CalendarEvent>,
}

#[derive(Template)]
#[template(path = "calendar.html")]
pub struct CalendarTemplate {
    pub app_name: String,
    pub username: String,
    pub tabs: Vec<TabLink>,
    /// "June 2026" display string
    pub month_label: String,
    /// "2026-05" link for previous month
    pub prev_ym: String,
    /// "2026-07" link for next month
    pub next_ym: String,
    /// 35 or 42 cells, week starts Monday
    pub days: Vec<CalendarDay>,
}