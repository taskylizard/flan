use crate::state::AppState;
use axum::{routing::post, Router};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

pub mod register;

#[derive(OpenApi)]
#[openapi(paths(register::register_user_handler))]
pub struct UserApi;

pub fn router() -> Router<AppState> {
    Router::new().route("/register", post(register::register_user_handler))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterUserRequest {
    /// Username to register
    #[schema(example = "tasky")]
    pub username: String,
    /// Admin key
    #[schema(example = "supersecretadminkey")]
    pub admin_key: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterUserResponse {
    /// Registered user's username
    #[schema(example = "tasky")]
    pub username: String,
    /// Registered user's access key
    #[schema(example = "tasky_hshsuwvb2u2ywv2v2bwb")]
    pub key: String,
}
