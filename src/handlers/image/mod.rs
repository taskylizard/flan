use crate::state::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

pub mod delete;
pub mod get;
pub mod list;
pub mod upload;

#[derive(OpenApi)]
#[openapi(paths(
    upload::upload_image_handler,
    list::list_images_handler,
    get::get_image_handler,
    delete::delete_image_handler
))]
pub struct ImageApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload::upload_image_handler))
        .route("/list", get(list::list_images_handler))
        .route("/:file_id", get(get::get_image_handler))
        .route("/:file_id", delete(delete::delete_image_handler))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UploadImageResponse {
    /// Image ID
    #[schema(example = "4c0bc20d-c2f2-4f8f-9bc7-c92bb26eda7d")]
    pub file_id: String,
    /// Image URL
    #[schema(example = "https://example.com/images/image_id")]
    pub url: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ImageInfo {
    /// Image ID
    pub file_id: String,
    /// Image URL
    pub url: String,
    /// Image creation date
    pub created_at: DateTime<Utc>,
    /// Whether the image is favorited
    pub favorite: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListImagesResponse {
    /// List of images
    #[schema(
        example = r#"[{"file_id": "image_id", "url": "https://example.com/images/image_id", "created_at": "2023-01-01T00:00:00Z", "favorite": true}]"#
    )]
    pub images: Vec<ImageInfo>,
}
