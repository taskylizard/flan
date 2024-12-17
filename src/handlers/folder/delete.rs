use crate::db::{folder, user};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use tracing::error;

#[derive(Debug)]
pub enum DeleteFolderError {
    InvalidCredentials,
    FolderNotFound,
    NotAuthorized,
    DatabaseError(String),
}

impl From<DeleteFolderError> for StatusCode {
    fn from(error: DeleteFolderError) -> StatusCode {
        match error {
            DeleteFolderError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            DeleteFolderError::FolderNotFound => StatusCode::NOT_FOUND,
            DeleteFolderError::NotAuthorized => StatusCode::FORBIDDEN,
            DeleteFolderError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn validate_user(
    state: &AppState,
    username: &str,
    key: &str,
) -> Result<String, DeleteFolderError> {
    let user = state
        .db
        .user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map_err(|e| DeleteFolderError::DatabaseError(e.to_string()))?;

    match user {
        Some(user) if user.access_key == key => Ok(user.id),
        _ => Err(DeleteFolderError::InvalidCredentials),
    }
}

async fn verify_folder_ownership(
    state: &AppState,
    folder_id: &str,
    user_id: &str,
) -> Result<(), DeleteFolderError> {
    let folder = state
        .db
        .folder()
        .find_first(vec![
            folder::id::equals(folder_id.to_string()),
            folder::user_id::equals(user_id.to_string()),
        ])
        .exec()
        .await
        .map_err(|e| DeleteFolderError::DatabaseError(e.to_string()))?;

    match folder {
        Some(_) => Ok(()),
        None => Err(DeleteFolderError::FolderNotFound),
    }
}

/// Delete a folder
#[utoipa::path(
    delete,
    path = "/delete/:folder_id",
    tag = "Folder",
    responses(
        (status = 204, description = "Folder deleted successfully"),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Folder not found"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn delete_folder_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(folder_id): Path<String>,
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

    // Verify folder ownership
    if let Err(e) = verify_folder_ownership(&state, &folder_id, &user_id).await {
        return Err(StatusCode::from(e));
    }

    // Delete folder
    state
        .db
        .folder()
        .delete(folder::id::equals(folder_id))
        .exec()
        .await
        .map_err(|e| {
            error!("Failed to delete folder from database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}
