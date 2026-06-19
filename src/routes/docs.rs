use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::health_controller::health_check,
        crate::controllers::auth_controller::register,
        crate::controllers::auth_controller::login,
        crate::controllers::item_controller::list_items,
        crate::controllers::item_controller::get_item,
        crate::controllers::item_controller::create_item,
        crate::controllers::item_controller::delete_item
    ),
    components(schemas(
        crate::models::health::HealthResponse,
        crate::models::user::RegisterRequest,
        crate::models::user::LoginRequest,
        crate::models::user::User,
        crate::models::auth::AuthResponse,
        crate::models::item::Item,
        crate::models::item::CreateItemRequest
    )),
    tags(
        (name = "health", description = "Health endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "items", description = "Item endpoints")
    )
)]
pub struct ApiDoc;

pub fn routes() -> Router<AppState> {
    Router::new().merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
}
