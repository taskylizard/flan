use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RegisterUserRequest {
    pub username: String,
    pub admin_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterUserResponse {
    pub username: String,
    pub key: String,
}
