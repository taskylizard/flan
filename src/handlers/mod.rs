use axum::{
    routing::{delete, get, post},
    Router,
};

pub mod delete_image;
pub mod get_image;
pub mod health_check;
pub mod list_images;
pub mod register_user;
pub mod upload_image;
use crate::state::AppState;

pub fn create_router() -> Router<AppState> {
    let api_router = Router::new()
        .route("/health", get(health_check::health_handler))
        .route("/register", post(register_user::register_user_handler))
        .route(
            "/delete/:file_id",
            delete(delete_image::delete_image_handler),
        )
        .route("/upload", post(upload_image::upload_image_handler))
        .route("/list", get(list_images::list_images_handler));

    let images_router = Router::new().route("/:file_id", get(get_image::get_image_handler));

    Router::new()
        .nest("/api", api_router)
        .nest("/images", images_router)
}
