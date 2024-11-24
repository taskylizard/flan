use crate::db::{image, user};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use tracing::error;

#[derive(Debug)]
pub enum DeleteImageError {
    InvalidCredentials,
    ImageNotFound,
    NotAuthorized,
    DatabaseError(String),
    StorageError(String),
}

impl From<DeleteImageError> for StatusCode {
    fn from(error: DeleteImageError) -> StatusCode {
        match error {
            DeleteImageError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            DeleteImageError::ImageNotFound => StatusCode::NOT_FOUND,
            DeleteImageError::NotAuthorized => StatusCode::FORBIDDEN,
            DeleteImageError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteImageError::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn validate_user(
    db: &AppState,
    username: &str,
    key: &str,
) -> Result<String, DeleteImageError> {
    let user = db
        .db
        .user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map_err(|e| DeleteImageError::DatabaseError(e.to_string()))?;

    match user {
        Some(user) if user.key == key => Ok(user.id),
        _ => Err(DeleteImageError::InvalidCredentials),
    }
}

async fn verify_image_ownership(
    state: &AppState,
    file_id: &str,
    user_id: &str,
) -> Result<String, DeleteImageError> {
    let image = state
        .db
        .image()
        .find_first(vec![image::file_id::equals(file_id.to_string())])
        .exec()
        .await
        .map_err(|e| DeleteImageError::DatabaseError(e.to_string()))?;

    match image {
        Some(image) if image.user_id == user_id => Ok(image.id),
        Some(_) => Err(DeleteImageError::NotAuthorized),
        None => Err(DeleteImageError::ImageNotFound),
    }
}

pub async fn delete_image_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
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

    // Verify image ownership
    let image_id = match verify_image_ownership(&state, &file_id, &user_id).await {
        Ok(id) => id,
        Err(e) => return Err(StatusCode::from(e)),
    };

    // Find the object in the bucket with the file_id prefix
    let objects = state
        .bucket
        .list(file_id.to_string(), None)
        .await
        .map_err(|e| {
            error!("Failed to list objects: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get the first object that matches our file_id
    if let Some(list) = objects.first() {
        if let Some(object) = list.contents.first() {
            // Delete from storage
            state.bucket.delete_object(&object.key).await.map_err(|e| {
                error!("Failed to delete object from storage: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    // Delete from database
    state
        .db
        .image()
        .delete(image::id::equals(image_id))
        .exec()
        .await
        .map_err(|e| {
            error!("Failed to delete image from database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}
