use super::AssociateTagRequest;
use crate::{
    db::{image, tag},
    handlers::tag::validate_user,
    state::AppState,
};
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
};
use tracing::error;

/// Associate a tag with an image
#[utoipa::path(
    post,
    path = "/associate/:image_id",
    tag = "Tag",
    request_body = AssociateTagRequest,
    responses(
        (status = 204, description = "Tag associated successfully"),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Image or tag not found"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn associate_tag_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(image_id): Path<String>,
    Json(payload): Json<AssociateTagRequest>,
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
    let user_id = validate_user(&state, username, key).await?;

    // Verify image ownership
    let image = state
        .db
        .image()
        .find_first(vec![
            image::id::equals(image_id.clone()),
            image::user_id::equals(user_id),
        ])
        .exec()
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if image.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Associate tag with image
    state
        .db
        .image()
        .update(
            image::id::equals(image_id),
            vec![image::tags::connect(vec![tag::id::equals(payload.tag_id)])],
        )
        .exec()
        .await
        .map_err(|e| {
            error!("Failed to associate tag with image: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}
