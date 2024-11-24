use crate::db::{image, user};
use crate::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use common::list::{ImageInfo, ListImagesResponse};
use tracing::error;

#[derive(Debug)]
pub enum ListImagesError {
    InvalidCredentials,
    DatabaseError(String),
}

impl From<ListImagesError> for StatusCode {
    fn from(error: ListImagesError) -> StatusCode {
        match error {
            ListImagesError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            ListImagesError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn validate_user(
    db: &AppState,
    username: &str,
    key: &str,
) -> Result<String, ListImagesError> {
    let user = db
        .db
        .user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map_err(|e| ListImagesError::DatabaseError(e.to_string()))?;

    match user {
        Some(user) if user.key == key => Ok(user.id),
        _ => Err(ListImagesError::InvalidCredentials),
    }
}

pub async fn list_images_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ListImagesResponse>, StatusCode> {
    // Extract username and key from headers
    let username = headers
        .get("X-Username")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let key = headers
        .get("X-Access-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate user and get user ID
    let user_id = match validate_user(&state, username, key).await {
        Ok(id) => id,
        Err(e) => return Err(StatusCode::from(e)),
    };

    // Get all images for the user
    let images = state
        .db
        .image()
        .find_many(vec![image::user_id::equals(user_id)])
        .exec()
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Transform the images into the response format
    let images = images
        .into_iter()
        .map(|img| ImageInfo {
            file_id: img.file_id.to_string(),
            url: format!("/images/{}", img.file_id),
            created_at: img.created_at.into(),
        })
        .collect();

    Ok(Json(ListImagesResponse { images }))
}
