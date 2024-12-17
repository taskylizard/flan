use super::{CreateOrUpdateFolderResponse, UpdateFolderRequest};
use crate::db::{folder, user};
use crate::state::AppState;
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
};
use tracing::error;

#[derive(Debug)]
pub enum UpdateFolderError {
    InvalidCredentials,
    FolderNotFound,
    NotAuthorized,
    DatabaseError(String),
}

impl From<UpdateFolderError> for StatusCode {
    fn from(error: UpdateFolderError) -> StatusCode {
        match error {
            UpdateFolderError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            UpdateFolderError::FolderNotFound => StatusCode::NOT_FOUND,
            UpdateFolderError::NotAuthorized => StatusCode::FORBIDDEN,
            UpdateFolderError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn validate_user(
    state: &AppState,
    username: &str,
    key: &str,
) -> Result<String, UpdateFolderError> {
    let user = state
        .db
        .user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map_err(|e| UpdateFolderError::DatabaseError(e.to_string()))?;

    match user {
        Some(user) if user.access_key == key => Ok(user.id),
        _ => Err(UpdateFolderError::InvalidCredentials),
    }
}

async fn verify_folder_ownership(
    state: &AppState,
    folder_id: &str,
    user_id: &str,
) -> Result<(), UpdateFolderError> {
    let folder = state
        .db
        .folder()
        .find_first(vec![
            folder::id::equals(folder_id.to_string()),
            folder::user_id::equals(user_id.to_string()),
        ])
        .exec()
        .await
        .map_err(|e| UpdateFolderError::DatabaseError(e.to_string()))?;

    match folder {
        Some(_) => Ok(()),
        None => Err(UpdateFolderError::FolderNotFound),
    }
}

/// Update a folder
#[utoipa::path(
    post,
    path = "/update/:folder_id",
    tag = "Folder",
    responses(
        (status = 200, description = "Folder updated successfully", body = CreateOrUpdateFolderResponse),
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
pub async fn update_folder_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(folder_id): Path<String>,
    Json(payload): Json<UpdateFolderRequest>,
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

    // Verify folder ownership
    if let Err(e) = verify_folder_ownership(&state, &folder_id, &user_id).await {
        return Err(StatusCode::from(e));
    }

    // Update folder
    let mut update_params = vec![];
    if let Some(name) = payload.name {
        update_params.push(folder::name::set(name));
    }
    if let Some(description) = payload.description {
        update_params.push(folder::description::set(Some(description)));
    }
    if let Some(pinned) = payload.pinned {
        update_params.push(folder::pinned::set(pinned));
    }

    let updated_folder = state
        .db
        .folder()
        .update(folder::id::equals(folder_id), update_params)
        .exec()
        .await
        .map_err(|e| {
            error!("Failed to update folder in database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(CreateOrUpdateFolderResponse {
        id: updated_folder.id,
        name: updated_folder.name,
        description: updated_folder.description,
        pinned: updated_folder.pinned,
    }))
}
