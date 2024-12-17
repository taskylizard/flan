use crate::db::user;
use crate::state::AppState;
use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use tracing::error;
use uuid::Uuid;

use super::UploadImageResponse;

#[derive(Debug)]
pub enum UploadError {
    InvalidCredentials,
    DatabaseError(String),
    InvalidFile,
    StorageError,
}

impl From<UploadError> for StatusCode {
    fn from(error: UploadError) -> StatusCode {
        match error {
            UploadError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UploadError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UploadError::InvalidFile => StatusCode::BAD_REQUEST,
            UploadError::StorageError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn validate_user(db: &AppState, username: &str, key: &str) -> Result<String, UploadError> {
    let user = db
        .db
        .user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map_err(|e| UploadError::DatabaseError(e.to_string()))?;

    match user {
        Some(user) if user.access_key == key => Ok(user.id),
        _ => Err(UploadError::InvalidCredentials),
    }
}

/// Upload an image
#[utoipa::path(
    post,
    path = "/upload",
    request_body(content_type = "multipart/form-data"), 
    responses(
        (status = 200, description = "Image upload response", body = UploadImageResponse),
        (status = 400, description = "Invalid file"),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("access_key" = [])
    ),
    tag = "Image"
)]
pub async fn upload_image_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<UploadImageResponse>, StatusCode> {
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

    // Handle file upload
    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        // Add debug logging for field name
        tracing::debug!("Received field name: {:?}", field.name());

        let file_name = field
            .file_name()
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();

        let content_type = field
            .content_type()
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();

        tracing::debug!("Content type: {}", content_type);

        // More lenient content type check
        if !content_type.starts_with("image/") && !content_type.contains("octet-stream") {
            tracing::error!("Invalid content type: {}", content_type);
            return Err(StatusCode::BAD_REQUEST);
        }

        let data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Generate a unique file ID
        let file_id = Uuid::new_v4().to_string();
        let extension = file_name.split('.').last().unwrap_or("jpg").to_lowercase();
        let object_name = format!("{}.{}", file_id, extension);

        tracing::debug!("Uploading object: {}", object_name);

        // Upload to storage
        state
            .bucket
            .put_object(&object_name, &data)
            .await
            .map_err(|e| {
                tracing::error!("Storage error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Create image record in database
        match state
            .db
            .image()
            .create(file_id.clone(), user::id::equals(user_id), vec![])
            .exec()
            .await
        {
            Ok(_) => {
                let url = format!("/images/{}", file_id);
                Ok(Json(UploadImageResponse { file_id, url }))
            }
            Err(e) => {
                error!("Failed to create image record: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        tracing::error!("No file field found in multipart form");
        Err(StatusCode::BAD_REQUEST)
    }
}
