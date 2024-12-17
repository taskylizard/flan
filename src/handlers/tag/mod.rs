use crate::{db::user, state::AppState};
use axum::{
    http::StatusCode,
    routing::{delete, post},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

pub mod associate;
pub mod create;
pub mod delete;

#[derive(OpenApi)]
#[openapi(paths(
    create::create_tag_handler,
    associate::associate_tag_handler,
    delete::delete_tag_handler
))]
pub struct TagApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", post(create::create_tag_handler))
        .route(
            "/associate/:image_id",
            post(associate::associate_tag_handler),
        )
        .route("/delete/:tag_id", delete(delete::delete_tag_handler))
}

/// Create a tag
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTagRequest {
    /// Tag name
    #[schema(example = "awesome")]
    pub name: String,
}

/// Response for creating a tag
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateTagResponse {
    /// Tag ID
    #[schema(example = "tag_id")]
    pub id: String,
    /// Tag name
    #[schema(example = "awesome")]
    pub name: String,
}

/// Associate a tag with an image
#[derive(Debug, Deserialize, ToSchema)]
pub struct AssociateTagRequest {
    /// Tag ID
    #[schema(example = "tag_id")]
    pub tag_id: String,
}

/// Response for listing tags
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListTagsResponse {
    /// List of tag IDs
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum TagError {
    InvalidCredentials,
    DatabaseError(String),
    TagNotFound,
    ImageNotFound,
    NotAuthorized,
}

impl From<TagError> for StatusCode {
    fn from(error: TagError) -> StatusCode {
        match error {
            TagError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            TagError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TagError::TagNotFound => StatusCode::NOT_FOUND,
            TagError::ImageNotFound => StatusCode::NOT_FOUND,
            TagError::NotAuthorized => StatusCode::FORBIDDEN,
        }
    }
}

pub async fn validate_user(
    state: &AppState,
    username: &str,
    key: &str,
) -> Result<String, TagError> {
    let user = state
        .db
        .user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map_err(|e| TagError::DatabaseError(e.to_string()))?;

    match user {
        Some(user) if user.access_key == key => Ok(user.id),
        _ => Err(TagError::InvalidCredentials),
    }
}
