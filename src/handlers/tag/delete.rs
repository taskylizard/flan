use crate::{db::tag, handlers::tag::validate_user, state::AppState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use tracing::error;

/// Delete a tag
#[utoipa::path(
    delete,
    path = "/delete/:tag_id",
    tag = "Tag",
    responses(
        (status = 204, description = "Tag deleted successfully"),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Tag not found"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn delete_tag_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tag_id): Path<String>,
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

    // Validate user
    let _ = validate_user(&state, username, key).await?;

    // Delete tag
    state
        .db
        .tag()
        .delete(tag::id::equals(tag_id))
        .exec()
        .await
        .map_err(|e| {
            error!("Failed to delete tag: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}
