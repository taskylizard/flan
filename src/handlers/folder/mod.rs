use crate::{db::user, state::AppState};
use axum::{
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

pub mod create;
pub mod delete;
pub mod list;
pub mod update;

#[derive(OpenApi)]
#[openapi(paths(
    create::create_folder_handler,
    list::list_folders_handler,
    update::update_folder_handler,
    delete::delete_folder_handler
))]
pub struct FolderApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", post(create::create_folder_handler))
        .route("/list", get(list::list_folders_handler))
        .route("/update/:folder_id", post(update::update_folder_handler))
        .route("/delete/:folder_id", delete(delete::delete_folder_handler))
}

#[derive(Debug)]
pub enum FolderError {
    InvalidCredentials,
    DatabaseError(String),
}

impl From<FolderError> for StatusCode {
    fn from(error: FolderError) -> StatusCode {
        match error {
            FolderError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            FolderError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn validate_user(
    db: &AppState,
    username: &str,
    key: &str,
) -> Result<String, FolderError> {
    let user = db
        .db
        .user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map_err(|e| FolderError::DatabaseError(e.to_string()))?;

    match user {
        Some(user) if user.access_key == key => Ok(user.id),
        _ => Err(FolderError::InvalidCredentials),
    }
}

/// List folders
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListFoldersResponse {
    /// List of folder IDs
    pub folders: Vec<String>,
}

/// Create a folder
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    /// Folder name
    #[schema(example = "Folder name")]
    pub name: String,
    /// Folder description
    #[schema(example = "Folder description")]
    pub description: Option<String>,
    /// Whether the folder is pinned
    #[schema(example = "true")]
    pub pinned: Option<bool>,
}

/// Response for creating or updating a folder
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateOrUpdateFolderResponse {
    /// Folder ID
    #[schema(example = "folder_id")]
    pub id: String,
    /// Folder name
    #[schema(example = "Folder name")]
    pub name: String,
    /// Folder description
    #[schema(example = "Folder description")]
    pub description: Option<String>,
    /// Whether the folder is pinned
    #[schema(example = "true")]
    pub pinned: bool,
}

/// Update a folder
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFolderRequest {
    /// Folder name
    #[schema(example = "Folder name")]
    pub name: Option<String>,
    /// Folder description
    #[schema(example = "Folder description")]
    pub description: Option<String>,
    /// Whether the folder is pinned
    #[schema(example = "true")]
    pub pinned: Option<bool>,
}
