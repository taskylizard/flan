use super::{validate_user, CreateFolderRequest, CreateOrUpdateFolderResponse};
use crate::db::{folder, user};
use crate::state::AppState;
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
};
use tracing::error;

/// Create a folder
#[utoipa::path(
    post,
    path = "/create",
    tag = "Folder",
    request_body = CreateFolderRequest,
    responses(
        (status = 200, description = "Folder created successfully", body = CreateOrUpdateFolderResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn create_folder_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateFolderRequest>,
) -> Result<Json<CreateOrUpdateFolderResponse>, StatusCode> {
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

    // Create folder
    let new_folder = state
        .db
        .folder()
        .create(
            payload.name,
            user::id::equals(user_id),
            vec![
                folder::description::set(payload.description),
                folder::pinned::set(payload.pinned.unwrap_or(false)),
            ],
        )
        .exec()
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(CreateOrUpdateFolderResponse {
        id: new_folder.id,
        name: new_folder.name,
        description: new_folder.description,
        pinned: new_folder.pinned,
    }))
}
