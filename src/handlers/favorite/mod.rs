use crate::db::{image, user};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde::Serialize;
use tracing::error;
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(paths(favorite_image_handler, unfavorite_image_handler))]
pub struct FavoriteApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mark/:image_id", post(favorite_image_handler))
        .route("/unmark/:image_id", post(unfavorite_image_handler))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FavoriteResponse {
    /// Image ID
    #[schema(example = "image_id")]
    image_id: String,
    /// Whether the image is favorited
    #[schema(example = "true")]
    favorite: bool,
}

#[derive(Debug)]
pub enum FavoriteError {
    InvalidCredentials,
    DatabaseError(String),
    ImageNotFound,
    NotAuthorized,
}

impl From<FavoriteError> for StatusCode {
    fn from(error: FavoriteError) -> StatusCode {
        match error {
            FavoriteError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            FavoriteError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FavoriteError::ImageNotFound => StatusCode::NOT_FOUND,
            FavoriteError::NotAuthorized => StatusCode::FORBIDDEN,
        }
    }
}

async fn validate_user(
    state: &AppState,
    username: &str,
    key: &str,
) -> Result<String, FavoriteError> {
    let user = state
        .db
        .user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map_err(|e| FavoriteError::DatabaseError(e.to_string()))?;

    match user {
        Some(user) if user.access_key == key => Ok(user.id),
        _ => Err(FavoriteError::InvalidCredentials),
    }
}

/// Mark an image as favorite
#[utoipa::path(
    post,
    path = "/mark/:image_id",
    tag = "Favorite",
    responses(
        (status = 200, description = "Image successfully marked", body = FavoriteResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Image not found"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn favorite_image_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(image_id): Path<String>,
) -> Result<Json<FavoriteResponse>, StatusCode> {
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

    // Update image favorite status
    let updated_image = state
        .db
        .image()
        .update_many(
            vec![
                image::id::equals(image_id.clone()),
                image::user_id::equals(user_id),
            ],
            vec![image::favorite::set(true)],
        )
        .exec()
        .await
        .map_err(|e| {
            error!("Failed to update image favorite status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if updated_image == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(FavoriteResponse {
        image_id,
        favorite: true,
    }))
}

/// Unmark an image as favorite
#[utoipa::path(
    post,
    path = "/unmark/:image_id",
    tag = "Favorite",
    responses(
        (status = 200, description = "Image successfully unmarked", body = FavoriteResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Image not found"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn unfavorite_image_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(image_id): Path<String>,
) -> Result<Json<FavoriteResponse>, StatusCode> {
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

    // Update image favorite status
    let updated_image = state
        .db
        .image()
        .update_many(
            vec![
                image::id::equals(image_id.clone()),
                image::user_id::equals(user_id),
            ],
            vec![image::favorite::set(false)],
        )
        .exec()
        .await
        .map_err(|e| {
            error!("Failed to update image favorite status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if updated_image == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(FavoriteResponse {
        image_id,
        favorite: false,
    }))
}
