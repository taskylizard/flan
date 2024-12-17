use super::validate_user;
use super::{CreateTagRequest, CreateTagResponse};
use crate::state::AppState;
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
};
use tracing::error;

/// Create a tag
#[utoipa::path(
    post,
    path = "/create",
    tag = "Tag",
    request_body = CreateTagRequest,
    responses(
        (status = 200, description = "Tag created successfully", body = CreateTagResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn create_tag_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateTagRequest>,
) -> Result<Json<CreateTagResponse>, StatusCode> {
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

    // Create tag
    let new_tag = state
        .db
        .tag()
        .create(payload.name, vec![])
        .exec()
        .await
        .map_err(|e| {
            error!("Failed to create tag: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(CreateTagResponse {
        id: new_tag.id,
        name: new_tag.name,
    }))
}
