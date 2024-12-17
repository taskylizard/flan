use super::{validate_user, ListFoldersResponse};
use crate::db::folder;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use tracing::error;

/// List folders
#[utoipa::path(
    get,
    path = "/list",
    tag = "Folder",
    responses(
        (status = 200, description = "List of folders", body = ListFoldersResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn list_folders_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ListFoldersResponse>, StatusCode> {
    let username = headers
        .get("X-Username")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let key = headers
        .get("X-Access-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user_id = match validate_user(&state, username, key).await {
        Ok(id) => id,
        Err(e) => return Err(StatusCode::from(e)),
    };

    let data = state
        .db
        .folder()
        .find_many(vec![folder::user_id::equals(user_id)])
        .exec()
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let folders = data.into_iter().map(|folder| folder.id).collect();

    Ok(Json(ListFoldersResponse { folders }))
}
